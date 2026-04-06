const targetDir = join(process.cwd(), directory);
if (!existsSync(targetDir)) {
    mkdirSync(targetDir, { recursive: true });
}

const srcDir = join(targetDir, "src");
if (!existsSync(srcDir)) mkdirSync(srcDir);

writeFileSync(
    join(targetDir, "package.json"),
    JSON.stringify(
        {
            name: "my-alloy-app",
            version: "1.0.0",
            dependencies: { "@alloyscript/engine": "*" },
            devDependencies: { "@alloyscript/cli": "*" },
        },
        null,
        2,
    ),
);

writeFileSync(
    join(targetDir, "tsconfig.json"),
    JSON.stringify(
        {
            compilerOptions: {
                target: "ES2022",
                module: "ESNext",
                moduleResolution: "bundler",
                strict: true,
                jsx: "react-jsx",
            },
        },
        null,
        2,
    ),
);

writeFileSync(
    join(srcDir, "main.ts"),
    'console.log("Hello from AlloyScript!");\n',
);

console.log(`🚀 Project initialised in ${targetDir}`);