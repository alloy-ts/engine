#include "parser.h"
#include <stdlib.h>
#include <string.h>

static Node *parse_pipeline(Lexer *l);

static Node *node_create(NodeType type) {
    Node *n = calloc(1, sizeof(Node));
    n->type = type;
    return n;
}

void node_free(Node *n) {
    if (!n) return;
    switch (n->type) {
        case NODE_SCRIPT:
            for (int i = 0; i < n->data.script.count; i++) node_free(n->data.script.commands[i]);
            free(n->data.script.commands);
            break;
        case NODE_PIPELINE:
            for (int i = 0; i < n->data.pipeline.count; i++) node_free(n->data.pipeline.stages[i]);
            free(n->data.pipeline.stages);
            break;
        case NODE_COMMAND:
            for (int i = 0; i < n->data.command.argc; i++) free(n->data.command.args[i]);
            free(n->data.command.args);
            for (int i = 0; i < n->data.command.redir_count; i++) node_free(n->data.command.redirects[i]);
            free(n->data.command.redirects);
            break;
        case NODE_REDIRECT:
            free(n->data.redirect.target);
            break;
        case NODE_ASSIGNMENT:
            free(n->data.assignment.name);
            free(n->data.assignment.value);
            break;
    }
    free(n);
}

static Node *parse_command(Lexer *l) {
    Token *t = lexer_next_token(l);
    if (t->type == TOKEN_EOF || t->type == TOKEN_NEWLINE || t->type == TOKEN_SEMICOLON) {
        token_free(t);
        return NULL;
    }

    if (t->type == TOKEN_ASSIGN) {
        Node *n = node_create(NODE_ASSIGNMENT);
        n->data.assignment.name = t->value;
        t->value = NULL;
        token_free(t);
        Token *val = lexer_next_token(l);
        n->data.assignment.value = val->value;
        val->value = NULL;
        token_free(val);
        return n;
    }

    Node *n = node_create(NODE_COMMAND);
    while (t->type == TOKEN_WORD || (t->type >= TOKEN_REDIR_IN && t->type <= TOKEN_REDIR_2TO1)) {
        if (t->type == TOKEN_WORD) {
            n->data.command.args = realloc(n->data.command.args, sizeof(char*) * (n->data.command.argc + 1));
            n->data.command.args[n->data.command.argc++] = t->value;
            t->value = NULL;
        } else {
            Node *r = node_create(NODE_REDIRECT);
            r->data.redirect.type = t->type;
            if (t->type != TOKEN_REDIR_1TO2 && t->type != TOKEN_REDIR_2TO1) {
                Token *target = lexer_next_token(l);
                r->data.redirect.target = target->value;
                target->value = NULL;
                token_free(target);
            }
            n->data.command.redirects = realloc(n->data.command.redirects, sizeof(Node*) * (n->data.command.redir_count + 1));
            n->data.command.redirects[n->data.command.redir_count++] = r;
        }
        token_free(t);
        t = lexer_next_token(l);
    }

    // Put back delimiter
    l->pos -= 1; // Simplification, in real parser we'd have lookahead
    token_free(t);

    return n;
}

static Node *parse_pipeline(Lexer *l) {
    Node *cmd = parse_command(l);
    if (!cmd) return NULL;

    Token *t = lexer_next_token(l);
    if (t->type == TOKEN_PIPE) {
        Node *p = node_create(NODE_PIPELINE);
        p->data.pipeline.stages = malloc(sizeof(Node*) * 2);
        p->data.pipeline.stages[0] = cmd;
        p->data.pipeline.count = 1;
        while (t->type == TOKEN_PIPE) {
            token_free(t);
            Node *next = parse_command(l);
            p->data.pipeline.stages = realloc(p->data.pipeline.stages, sizeof(Node*) * (p->data.pipeline.count + 1));
            p->data.pipeline.stages[p->data.pipeline.count++] = next;
            t = lexer_next_token(l);
        }
        l->pos -= 1;
        token_free(t);
        return p;
    }

    l->pos -= 1;
    token_free(t);
    return cmd;
}

Node *parser_parse(Lexer *l) {
    Node *root = node_create(NODE_SCRIPT);
    Node *n;
    while ((n = parse_pipeline(l))) {
        root->data.script.commands = realloc(root->data.script.commands, sizeof(Node*) * (root->data.script.count + 1));
        root->data.script.commands[root->data.script.count++] = n;
        Token *t = lexer_next_token(l);
        if (t->type == TOKEN_EOF) {
            token_free(t);
            break;
        }
        token_free(t);
    }
    return root;
}
