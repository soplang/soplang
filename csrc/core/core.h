/**
 * Soplang Core Module
 *
 * This module contains the core language components:
 * - Lexer: Tokenization of source code
 * - Parser: Parsing tokens into an abstract syntax tree
 * - AST: Abstract syntax tree representation
 * - Tokens: Token types and definitions
 */

#ifndef SOPLANG_CORE_H
#define SOPLANG_CORE_H

// Forward declarations
struct Token;
struct TokenType;
struct ASTNode;
struct NodeType;
struct Lexer;
struct Parser;

// Include component headers
#include "lexer.h"
#include "parser.h"
#include "tokens.h"
#include "ast.h"

#endif // SOPLANG_CORE_H
