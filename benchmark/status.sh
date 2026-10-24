#!/usr/bin/env bash
# Three-way status benchmark. All repositories are inputs; this never refreshes an index.
set -euo pipefail
exec python3 - "$@" <<'PYTHON'
import argparse, hashlib, json, math, os, statistics, struct, subprocess, sys, tempfile, time
from pathlib import Path

p = argparse.ArgumentParser(description="Read-only Linux status comparison (resource sampling is outside timing runs)")
for name in ('installed-libra', 'built-libra', 'git', 'libra-repo', 'git-repo', 'output'):
    p.add_argument('--' + name, required=True)
for name, default in [('warmup', 3), ('runs', 20), ('rounds', 3)]:
    p.add_argument('--' + name, type=int, default=default)
a = p.parse_args()
if a.warmup < 0 or a.runs < 1 or a.rounds < 1: p.error('invalid sample counts')
if not Path('/proc/self/status').exists(): p.error('Linux /proc is required for the separate process-tree memory sample')

def sha(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''): h.update(chunk)
    return h.hexdigest()

def index_info(root):
    root = Path(root)
    f = root / ('.libra/index' if (root / '.libra').is_dir() else '.git/index')
    raw = f.read_bytes()
    sig, version, count = struct.unpack('>4sII', raw[:12])
    if sig != b'DIRC' or version not in (2, 3): raise ValueError('benchmark needs a v2/v3 SHA-1 index fixture')
    # This benchmark deliberately rejects other formats rather than reporting false stat hits.
    if hashlib.sha1(raw[:-20]).digest() != raw[-20:]: raise ValueError('fixture index is not a verified SHA-1 index')
    pos = 12; zero = hits = 0; content = hashlib.sha256()
    for _ in range(count):
        start = pos; fields = struct.unpack('>10I', raw[pos:pos+40])
        flags = struct.unpack('>H', raw[pos+60:pos+62])[0]; pos += 62
        if flags & 0x4000: pos += 2
        end = raw.index(b'\0', pos); name = raw[pos:end]; pos = start + ((end-start+8)//8)*8
        zero += not any(fields[:4]); path = root / os.fsdecode(name)
        content.update(name + b'\0')
        try:
            st = path.lstat()
            hits += fields[:4] == (st.st_ctime_ns//10**9, st.st_ctime_ns%10**9, st.st_mtime_ns//10**9, st.st_mtime_ns%10**9) and fields[9] == st.st_size
            if path.is_symlink(): digest = hashlib.sha256(os.fsencode(os.readlink(path))).digest()
            elif path.is_file(): digest = bytes.fromhex(sha(path))
            else: digest = b'<directory>'
            content.update(digest)
        except FileNotFoundError: content.update(b'<missing>')
    return dict(index_sha256=sha(f), entries=count, zero_stat=zero, matching_stat=hits, tracked_content_sha256=content.hexdigest())

def metadata(binary, root):
    version = subprocess.check_output([binary, '--version'], text=True, stderr=subprocess.DEVNULL).strip()
    head = subprocess.check_output([binary, 'rev-parse', 'HEAD'], cwd=root, text=True, stderr=subprocess.DEVNULL).strip()
    return dict(binary=binary, binary_sha256=sha(binary), version=version, root=root, head=head, index=index_info(root))

def sample(binary, root):
    # wait4 gives per-invocation resources, not cumulative RUSAGE_CHILDREN deltas.
    # RSS from this fork can inherit Python's floor, so memory is sampled separately.
    with tempfile.TemporaryFile() as err:
        start = time.perf_counter_ns()
        proc = subprocess.Popen([binary, 'status'], cwd=root, stdout=subprocess.DEVNULL, stderr=err)
        _, status, usage = os.wait4(proc.pid, 0)
        elapsed = (time.perf_counter_ns()-start)/1e6
        proc.returncode = os.waitstatus_to_exitcode(status)
        if proc.returncode: raise RuntimeError(f'{binary} status failed with {proc.returncode}; sample rejected')
        err.seek(0); stderr = err.read()
    return dict(wall_ms=elapsed, user_ms=usage.ru_utime*1000, system_ms=usage.ru_stime*1000,
                voluntary=usage.ru_nvcsw, involuntary=usage.ru_nivcsw,
                stderr_bytes=len(stderr), stderr_sha256=hashlib.sha256(stderr).hexdigest())

def tree_memory(binary, root):
    with tempfile.TemporaryFile() as err:
        proc = subprocess.Popen([binary, 'status'], cwd=root, stdout=subprocess.DEVNULL, stderr=err)
        peak = 0; samples = 0; cpu = {}; peak_processes = 0
        while proc.poll() is None:
            todo = [proc.pid]; seen = set(); rss = 0
            while todo:
                pid = todo.pop()
                if pid in seen: continue
                seen.add(pid)
                try:
                    for task in Path(f'/proc/{pid}/task').iterdir():
                        try: todo.extend(map(int, (task/'children').read_text().split()))
                        except (OSError, ValueError): pass
                    ticks = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
                    cpu[pid] = (int(ticks[11]), int(ticks[12]))
                    stat = Path(f'/proc/{pid}/statm').read_text().split()
                    rss += int(stat[1]) * os.sysconf('SC_PAGE_SIZE') // 1024
                except (OSError, ValueError, IndexError): pass
            peak = max(peak, rss); peak_processes = max(peak_processes, len(seen)); samples += 1; time.sleep(0.005)
        if proc.returncode: raise RuntimeError('separate memory sample failed')
    return dict(tree_rss_kib_sampled_peak=peak, tree_processes_sampled_peak=peak_processes, samples=samples, interval_ms=5,
                tree_user_ms_sampled=sum(v[0] for v in cpu.values())*1000/os.sysconf('SC_CLK_TCK'),
                tree_system_ms_sampled=sum(v[1] for v in cpu.values())*1000/os.sysconf('SC_CLK_TCK'),
                note='Separate untimed run; RSS sum counts shared pages repeatedly and may miss brief peaks')

jobs = [('installed', str(Path(a.installed_libra).resolve()), str(Path(a.libra_repo).resolve())),
        ('built', str(Path(a.built_libra).resolve()), str(Path(a.libra_repo).resolve())),
        ('git', str(Path(a.git).resolve()), str(Path(a.git_repo).resolve()))]
result = dict(method='new process, warm filesystem, default status, alternating order; wait4 CPU covers parent and reaped children only; process-tree CPU/RSS sampled separately',
              warmup=a.warmup, runs=a.runs, rounds=[], inputs={k:metadata(b,r) for k,b,r in jobs})
result['matching_tracked_content'] = result['inputs']['built']['index']['tracked_content_sha256'] == result['inputs']['git']['index']['tracked_content_sha256']
result['matching_head'] = result['inputs']['built']['head'] == result['inputs']['git']['head']
out = Path(a.output)
# Refuse to overwrite an input by accident, including an output inside a measured repository.
resolved_out = out.resolve()
if any(resolved_out == Path(b) or resolved_out.is_relative_to(Path(r)) for _,b,r in jobs):
    p.error('output must be outside measured repositories and binaries')
for round_id in range(a.rounds):
    for _ in range(a.warmup):
        for _, b, r in jobs: sample(b, r)
    values = {k:[] for k,_,_ in jobs}
    for i in range(a.runs):
        order = jobs[(i+round_id)%3:] + jobs[:(i+round_id)%3]
        for k,b,r in order: values[k].append(sample(b,r))
    summary = {}
    for k,b,r in jobs:
        rows = values[k]
        summary[k] = {name:statistics.median(v[name] for v in rows) for name in ('wall_ms','user_ms','system_ms','voluntary','involuntary')}
        summary[k]['p95_ms'] = sorted(v['wall_ms'] for v in rows)[math.ceil(len(rows)*0.95)-1]
        summary[k]['memory'] = tree_memory(b,r)
    ratio = summary['built']['wall_ms'] / summary['git']['wall_ms']
    result['rounds'].append(dict(round=round_id+1, samples=values, summary=summary, built_git_ratio=ratio))
    out.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(dict(round=round_id+1, summary=summary, built_git_ratio=ratio)), flush=True)
for k,b,r in jobs:
    after = index_info(r)
    if after != result['inputs'][k]['index']: raise RuntimeError('repository index or tracked content changed during benchmark')
result['inputs_unchanged'] = True
result['performance_target_met'] = result['matching_tracked_content'] and result['matching_head'] and all(r['built_git_ratio'] <= 5 for r in result['rounds'])
out.write_text(json.dumps(result, indent=2)+'\n')
PYTHON
