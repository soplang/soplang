/**
 * Soplang Token Definitions
 */

#ifndef SOPLANG_TOKENS_H
#define SOPLANG_TOKENS_H

// Token type enum
typedef enum {
    // Keywords
    DOOR,           // door (variable declaration)
    hawl,           // hawl (function definition)
    SOO_CELI,       // soo_celi (return)
    BANDHIG,            // bandhig (print)
    AKHRI,          // akhri (input)
    HADDII,         // haddii (if)
    HADDII_KALE,    // haddii_kale (else if)
    HADDII_KALENA,  // haddii_kalena (else)
    KU_CELI,        // ku_celi (for)
    INTA_AY,        // inta_ay (while)
    JOOJI,          // jooji (break)
    SII_WAD,        // sii_wad (continue)
    ISKU_DAY,       // isku_day (try)
    QABO,           // qabo (catch)
    KA_KEEN,        // ka_keen (import)
    FASALKA,        // fasalka (class)
    KA_DHAXAL,      // ka_dhaxal (extends)
    CUSUB,          // cusub (new)
    NAFTA,          // nafta (self/this)

    // Static types
    TIRO,           // tiro (number)
    QORAAL,         // qoraal (string)
    LABADARAN,      // labadaran (boolean)
    LIIS,           // liis (list/array)
    SHEY,           // shey (object)

    // Literals
    NUMBER,         // Number literal
    STRING,         // String literal
    IDENTIFIER,     // Variable or function name
    TRUE,           // true
    FALSE,          // false
    NULL,           // null

    // Operators
    PLUS,           // +
    MINUS,          // -
    STAR,           // *
    SLASH,          // /
    MODULO,         // %
    EQUAL,          // =
    NOT_EQUAL,      // !=
    GREATER,        // >
    LESS,           // <
    GREATER_EQUAL,  // >=
    LESS_EQUAL,     // <=
    AND,            // &&
    OR,             // ||
    NOT,            // !

    // Punctuation
    LEFT_PAREN,     // (
    RIGHT_PAREN,    // )
    LEFT_BRACE,     // {
    RIGHT_BRACE,    // }
    LEFT_BRACKET,   // [
    RIGHT_BRACKET,  // ]
    COMMA,          // ,
    DOT,            // .
    COLON,          // :
    SEMICOLON,      // ;

    // Other
    EOF             // End of file
} TokenType;

// Token structure
typedef struct {
    TokenType type;
    char* value;
    int line;
    int column;
} Token;

// Function declarations
Token* token_create(TokenType type, char* value, int line, int column);
void token_free(Token* token);
const char* token_type_to_string(TokenType type);

#endif // SOPLANG_TOKENS_H
