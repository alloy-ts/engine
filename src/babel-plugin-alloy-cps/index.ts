// src/babel-plugin-alloy-cps/index.ts
// CPS transform for async/await to ES5 callbacks

export default function(babel: any) {
  const { types: t } = babel;

  return {
    name: "alloy-cps",
    visitor: {
      Function(path: any) {
        if (!path.node.async) return;
        path.node.async = false;

        // CPS Rewrite: add __k parameter
        path.node.params.push(t.identifier("__k"));

        path.traverse({
          AwaitExpression(awaitPath: any) {
            const expr = awaitPath.node.argument;
            // Rewrite await to __alloy_dispatch
            awaitPath.replaceWith(
              t.callExpression(t.identifier("__alloy_dispatch"), [
                t.stringLiteral("await"),
                t.arrayExpression([expr]),
                t.identifier("__k")
              ])
            );
          },
          ReturnStatement(returnPath: any) {
            const arg = returnPath.node.argument || t.identifier("undefined");
            returnPath.replaceWith(
              t.callExpression(t.identifier("__k"), [t.nullLiteral(), arg])
            );
          },
          ThrowStatement(throwPath: any) {
             throwPath.replaceWith(
                 t.callExpression(t.identifier("__k"), [throwPath.node.argument])
             );
          }
        });
      }
    }
  };
}
