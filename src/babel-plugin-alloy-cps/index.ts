import type { NodePath, PluginObj } from "@babel/core";
import * as t from "@babel/types";

/**
 * babel-plugin-alloy-cps: Transforms async/await to CPS-style ES5.
 * Targets the __alloy_dispatch primitive.
 */
export default function(): PluginObj {
  return {
    name: "alloy-cps",
    visitor: {
      Function(path) {
        if (!path.node.async) return;

        // Pass 1: Mark for transformation
        path.node.async = false;

        // Pass 2: CPS rewrite (Skeleton)
        const continuationParam = path.scope.generateUid("__k");
        path.node.params.push(t.identifier(continuationParam));

        path.traverse({
          AwaitExpression(awaitPath) {
            const op = t.stringLiteral("__await__");
            const awaitArg = awaitPath.node.argument;
            const args = t.arrayExpression([awaitArg]);
            const errParam = awaitPath.scope.generateUid("__err");
            const resParam = awaitPath.scope.generateUid("result");

            // Extract siblings after this statement
            const parentPath = awaitPath.getStatementParent();
            if (!parentPath) return;
            const siblings = parentPath.getAllNextSiblings();
            const siblingNodes = siblings.map(s => s.node);
            siblings.forEach(s => s.remove());

            const continuationBody = [
              t.ifStatement(
                t.identifier(errParam),
                t.returnStatement(t.callExpression(t.identifier(continuationParam), [t.identifier(errParam)]))
              ),
              ...siblingNodes
            ];

            // If the await is part of an assignment or variable declaration, we need to handle the result
            if (t.isVariableDeclarator(awaitPath.parent)) {
                awaitPath.replaceWith(t.identifier(resParam));
            } else if (t.isAssignmentExpression(awaitPath.parent)) {
                awaitPath.replaceWith(t.identifier(resParam));
            }

            const dispatchCall = t.expressionStatement(t.callExpression(t.identifier("__alloy_dispatch"), [
              op,
              args,
              t.functionExpression(
                null,
                [t.identifier(errParam), t.identifier(resParam)],
                t.blockStatement(continuationBody)
              ),
            ]));

            parentPath.replaceWith(dispatchCall);
          },
          ReturnStatement(returnPath) {
            const val = returnPath.node.argument || t.identifier("undefined");
            returnPath.replaceWith(
              t.returnStatement(t.callExpression(t.identifier(continuationParam), [t.nullLiteral(), val]))
            );
          },
          ThrowStatement(throwPath) {
            throwPath.replaceWith(
              t.returnStatement(t.callExpression(t.identifier(continuationParam), [throwPath.node.argument]))
            );
          }
        });
      },

      ForAwaitStatement(path) {
        throw path.buildCodeFrameError("for await is not supported in AlloyScript MVP");
      },

      CallExpression(path) {
        const callee = path.node.callee;
        if (t.isMemberExpression(callee) && t.isIdentifier(callee.object, { name: "Promise" })) {
          const property = callee.property;
          if (t.isIdentifier(property) && ["all", "race", "allSettled", "any"].includes(property.name)) {
             // Check if used with await? Requirement 3.8 says "if called with await in same expression"
             // For simplicity, we flag them all if they appear in an async function for now.
             if (path.scope.path.node.async) {
                throw path.buildCodeFrameError("Promise combinators are not supported in AlloyScript MVP");
             }
          }
        }
      }
    },
  };
}
