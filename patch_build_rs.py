import os

for root, dirs, files in os.walk("./crates"):
    for file in files:
        if file == "build.rs":
            path = os.path.join(root, file)
            with open(path, "r") as f:
                content = f.read()

            if 'join("..\\\\..\\\\..\\\\")' in content:
                print(f"Patching {path}")
                content = content.replace('.join("..\\\\..\\\\..\\\\")', '.parent().unwrap().parent().unwrap().parent().unwrap()')

                with open(path, "w") as f:
                    f.write(content)
