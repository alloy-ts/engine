import type { PluginObj, NodePath } from "@babel/core";
import * as t from "@babel/types";

/**
 * babel-plugin-alloy-cps
 *
 * Transforms async/await into CPS-style ES5 using __alloy_dispatch.
 */

export default function alloyCpsPlugin(): PluginObj {
  return {
    name: "alloy-cps",
    visitor: {
      Function(path) {
        if (!path.node.async) return;

        path.node.async = false;
        const kParam = path.scope.generateUidIdentifier("__k");
        path.node.params.push(kParam);

        const bodyPath = path.get("body") as NodePath<t.BlockStatement>;
        const newBody = transformStatements(bodyPath.get("body"), kParam);
        bodyPath.replaceWith(t.blockStatement(newBody));
      },

      ForAwaitStatement(path) {
        throw path.buildCodeFrameError("for await is not supported in AlloyScript MVP");
      },

      CallExpression(path) {
        const callee = path.node.callee;
        if (
          t.isMemberExpression(callee) &&
          t.isIdentifier(callee.object, { name: "Promise" }) &&
          t.isIdentifier(callee.property) &&
          ["all", "race", "allSettled", "any"].includes(callee.property.name)
        ) {
          let hasAwait = false;
          path.traverse({
            AwaitExpression() {
              hasAwait = true;
            },
          });
          if (hasAwait) {
            throw path.buildCodeFrameError(
              `Promise.${callee.property.name} with await is not supported in AlloyScript MVP`,
            );
          }
        }
      },
    },
  };
}

function transformStatements(
  statementPaths: NodePath<t.Statement>[],
  k: t.Expression,
): t.Statement[] {
  if (statementPaths.length === 0) {
    return [t.returnStatement(t.callExpression(k, [t.nullLiteral(), t.identifier("undefined")]))];
  }

  const [firstPath, ...restPaths] = statementPaths;
  const firstNode = firstPath.node;

  // Handle Return
  if (t.isReturnStatement(firstNode)) {
    const val = firstNode.argument || t.identifier("undefined");
    return [t.returnStatement(t.callExpression(k, [t.nullLiteral(), val]))];
  }

  // Handle Throw
  if (t.isThrowStatement(firstNode)) {
    return [t.returnStatement(t.callExpression(k, [firstNode.argument]))];
  }

  // Find the first await in the current statement (not in nested functions)
  let firstAwaitPath: NodePath<t.AwaitExpression> | null = null;
  firstPath.traverse({
    AwaitExpression(p) {
      if (!firstAwaitPath && p.scope.getFunctionParent() === firstPath.scope.getFunctionParent()) {
        firstAwaitPath = p;
      }
    },
  });

  if (firstAwaitPath) {
    const awaitExpr = (firstAwaitPath as NodePath<t.AwaitExpression>).node;
    const tempId = firstPath.scope.generateUidIdentifier("__res");
    (firstAwaitPath as NodePath<t.AwaitExpression>).replaceWith(tempId);

    const nextK = t.functionExpression(
        null,
        [t.identifier("__err"), tempId],
        t.blockStatement([
            t.ifStatement(t.identifier("__err"), t.returnStatement(t.callExpression(k, [t.identifier("__err")]))),
            ...transformStatements([firstPath, ...restPaths], k)
        ])
    );

    return [
      t.returnStatement(
        t.callExpression(t.identifier("__alloy_dispatch"), [
          t.stringLiteral("__await__"),
          t.arrayExpression([awaitExpr.argument]),
          nextK,
        ]),
      ),
    ];
  }

  // No await in this statement, but might be a block with awaits
  if (t.isBlockStatement(firstNode)) {
      const nextK = createContinuation(firstPath, restPaths, k);
      return transformStatements((firstPath as NodePath<t.BlockStatement>).get("body"), nextK);
  }

  if (t.isIfStatement(firstNode) || t.isWhileStatement(firstNode) || t.isTryStatement(firstNode)) {
      return handleControlFlow(firstPath as NodePath<t.IfStatement | t.WhileStatement | t.TryStatement>, restPaths, k);
  }

  return [firstNode, ...transformStatements(restPaths, k)];
}

function handleControlFlow(
    path: NodePath<t.IfStatement | t.WhileStatement | t.TryStatement>,
    restPaths: NodePath<t.Statement>[],
    k: t.Expression
): t.Statement[] {
    const node = path.node;
    const nextK = createContinuation(path, restPaths, k);

    if (t.isIfStatement(node)) {
        const consequent = t.isBlockStatement(node.consequent)
          ? transformStatements((path.get("consequent") as NodePath<t.BlockStatement>).get("body"), nextK)
          : transformStatements([path.get("consequent") as NodePath<t.Statement>], nextK);

        const alternate = node.alternate
          ? t.isBlockStatement(node.alternate)
            ? transformStatements((path.get("alternate") as NodePath<t.BlockStatement>).get("body"), nextK)
            : transformStatements([path.get("alternate") as NodePath<t.Statement>], nextK)
          : [t.returnStatement(t.callExpression(nextK, [t.nullLiteral(), t.identifier("undefined")]))];

        return [t.ifStatement(node.test, t.blockStatement(consequent), t.blockStatement(alternate))];
    }

    if (t.isWhileStatement(node)) {
        const loopId = path.scope.generateUidIdentifier("__loop");
        const loopBody = t.isBlockStatement(node.body)
          ? (path.get("body") as NodePath<t.BlockStatement>).get("body")
          : [path.get("body") as NodePath<t.Statement>];

        const loopCont = t.functionExpression(
            loopId,
            [],
            t.blockStatement([
                t.ifStatement(
                    node.test,
                    t.blockStatement(transformStatements(loopBody, loopId)),
                    t.blockStatement([
                        t.returnStatement(t.callExpression(nextK, [t.nullLiteral(), t.identifier("undefined")]))
                    ])
                )
            ])
        );

        return [
            t.variableDeclaration("var", [t.variableDeclarator(loopId, loopCont)]),
            t.expressionStatement(t.callExpression(loopId, [])),
            t.returnStatement()
        ];
    }

    if (t.isTryStatement(node)) {
        const catchClause = node.handler;
        const catchParam = catchClause ? (catchClause.param as t.Identifier) : path.scope.generateUidIdentifier("e");

        const tryNextK = t.functionExpression(
            null,
            [t.identifier("__err"), t.identifier("__res")],
            t.blockStatement([
                t.ifStatement(
                    t.identifier("__err"),
                    t.blockStatement([
                        t.variableDeclaration("var", [t.variableDeclarator(catchParam, t.identifier("__err"))]),
                        ...(catchClause ? transformStatements((path.get("handler.body") as NodePath<t.BlockStatement>).get("body"), nextK) : [t.returnStatement(t.callExpression(nextK, [t.identifier("__err")]))])
                    ]),
                    t.blockStatement([
                        t.returnStatement(t.callExpression(nextK, [t.nullLiteral(), t.identifier("__res")]))
                    ])
                )
            ])
        );

        return transformStatements((path.get("block") as NodePath<t.BlockStatement>).get("body"), tryNextK);
    }

    return [];
}

function createContinuation(
  path: NodePath<t.Statement>,
  restPaths: NodePath<t.Statement>[],
  k: t.Expression
): t.FunctionExpression {
  const errId = path.scope.generateUidIdentifier("__err");
  const resId = path.scope.generateUidIdentifier("__res");

  return t.functionExpression(
    null,
    [errId, resId],
    t.blockStatement([
      t.ifStatement(errId, t.returnStatement(t.callExpression(k, [errId]))),
      ...transformStatements(restPaths, k),
    ]),
  );
}
