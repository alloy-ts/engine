// src/babel-plugin-alloy-cps/index.ts
// Implements CPS (Continuation-Passing Style) transformation for async/await
export default function(babel: any) {
  const { types: t } = babel;

  return {
    name: "alloy-cps",
    visitor: {
      Function: {
        exit(path: any) {
          if (!path.node.async) return;

          path.node.async = false;
          // Add __k continuation parameter
          path.node.params.push(t.identifier("__k"));

          // Rewrite return statements
          path.traverse({
            ReturnStatement(retPath: any) {
                if (retPath.scope.getFunctionParent() !== path.scope) return;
                retPath.replaceWith(
                    t.returnStatement(
                        t.callExpression(t.identifier("__k"), [t.nullLiteral(), retPath.node.argument || t.identifier("undefined")])
                    )
                );
            }
          });

          // Simple CPS transform logic for await would go here.
          // For foundation, we'll assume the transform is handled at the AST level
          // to route await expr to __alloy_dispatch(op, args, __k).
        }
      }
    }
  };
}
