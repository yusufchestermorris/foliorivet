"""Package local CLI candidates with complete collected notice inventories."""
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED
import hashlib, json

repo = Path(__file__).resolve().parents[2]
workspace = repo.parent
dist = repo / 'dist'
rows = []

targets = [
    ('windows-x64', repo / 'target/debug/foliorivet-cli.exe', 'foliorivet-0.1.0-windows-x64-cli-review.zip', 'dev'),
    ('linux-x64', workspace / '.tooling/linux/target/debug/foliorivet-cli', 'foliorivet-0.1.0-linux-x64-cli-review.zip', 'dev'),
    ('windows-x64', dist / 'release-windows/foliorivet-cli.exe', 'foliorivet-0.1.0-windows-x64-cli.zip', 'release'),
    ('linux-x64', dist / 'release-linux/foliorivet-cli', 'foliorivet-0.1.0-linux-x64-cli.zip', 'release'),
]

for platform, exe, out_name, profile in targets:
    if not exe.is_file():
        continue
    output = dist / out_name
    with ZipFile(output, 'w', ZIP_DEFLATED) as z:
        z.write(exe, exe.name)
        for name in ['LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE']:
            z.write(repo / name, name)
        for directory, prefix in [
            (dist / 'cli-notices', 'licenses/dependencies'),
            (repo / 'bindings/python/python/foliorivet/licenses/language-data', 'licenses/language-data'),
        ]:
            if directory.is_dir():
                for f in directory.rglob('*'):
                    if f.is_file():
                        z.write(f, prefix + '/' + f.relative_to(directory).as_posix())
        z.writestr(
            'CANDIDATE.txt',
            f'FolioRivet 0.1.0 {profile}-profile review candidate. Based on WordCraft by the ArtCraft team. '
            'Not release-cleared. Separate-machine and final compatibility checks remain pending.\n'
        )
    rows.append({
        'file': output.name,
        'sha256': hashlib.sha256(output.read_bytes()).hexdigest(),
        'profile': profile,
        'release': 'HOLD'
    })

for wheel in sorted(dist.glob('*/*.whl')):
    prof = 'release' if 'release' in str(wheel.parent) else 'dev'
    rows.append({
        'file': wheel.relative_to(dist).as_posix(),
        'sha256': hashlib.sha256(wheel.read_bytes()).hexdigest(),
        'profile': prof,
        'release': 'HOLD'
    })

(dist / 'candidate-manifest.json').write_text(json.dumps(rows, indent=2) + '\n', encoding='utf-8')
print(json.dumps(rows, indent=2))
