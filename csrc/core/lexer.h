/**
 * Soplang Lexer
 *
 * Tokenizes source code into a stream of tokens.
 */

#ifndef SOPLANG_LEXER_H
#define SOPLANG_LEXER_H

#include "tokens.h"

// Lexer structure
typedef struct {
    char* source;     // The source code
    int position;     // Current position in the source
    int line;         // Current line number
    int column;       // Current column number
    char current_char; // Current character
} Lexer;

// Function declarations
Lexer* lexer_create(char* source);
void lexer_free(Lexer* lexer);
void lexer_advance(Lexer* lexer);
void lexer_skip_whitespace(Lexer* lexer);
int lexer_skip_comment(Lexer* lexer);
char lexer_peek(Lexer* lexer);
Token* lexer_tokenize_identifier(Lexer* lexer);
Token* lexer_tokenize_number(Lexer* lexer);
Token* lexer_tokenize_string(Lexer* lexer);
Token* lexer_next_token(Lexer* lexer);
Token** lexer_tokenize(Lexer* lexer, int* token_count);

#endif // SOPLANG_LEXER_H
