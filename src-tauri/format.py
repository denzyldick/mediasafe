with open('src/ml.rs') as f:
    lines = f.readlines()
indent = 0
for i, line in enumerate(lines):
    print(f"{i+1:3} [{indent:2}] {line.strip()[:40]}")
    indent += line.count('{') - line.count('}')
