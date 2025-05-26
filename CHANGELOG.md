# CHANGELOG

All notable changes to this project will be documented in this file.
## [Unreleased]
## [v2.0.0] - 2025-05-26

## (unreleased)

### Added

* **feat:** implement `kudhow()` for random number and list item selection [Mr Sharafdin]

* **feat:** add dherer() function to return length of lists, strings, and objects [Mr Sharafdin]

* **feat:** implement `jar()` to slice substrings by index [Mr Sharafdin]

* **feat:** implement `kudar()` for joining a list of strings [Mr Sharafdin]

* **feat:** implement `beddel()` to replace part of a string [Mr Sharafdin]

* **feat:** implement `bilow()` method to check string prefix [Mr Sharafdin]

* **feat:** implement `dhamaad()` to check string suffix match [Mr Sharafdin]

* **feat:** add `kor()` to round numbers up to the nearest integer [Mr Sharafdin]

* **feat:** implement leeyahay() method to check if a string contains a substring [Mr Sharafdin]

* **feat:** add `daji()` to round numbers down to nearest integer [Mr Sharafdin]

* **feat:** implement `qeybi()` method to split strings by delimiter [Mr Sharafdin]

* **feat:** implement `lamaane()` to return [key, value] pairs from walax [Mr Sharafdin]

* **feat:** add walax.qiime() method for getting all values from an object [Mr Sharafdin]

* **feat:** add `nadiifi()` method to clear all keys from a walax [Mr Sharafdin]

* **feat:** implement `nuqul()` for walax to create shallow object copies [Mr Sharafdin]

* **feat:** add 'raadso()' and support negative indexing in lists [Mr Sharafdin]

* **feat:** Implement aaddin() method in interpreter and builtins [Mr Sharafdin]

* **feat:** Implement aaddin() method for list transformations [Mr Sharafdin]

* **feat:** add `jar(start, end)` method to slice lists [Mr Sharafdin]

* **feat:** Enhance madoor implementation with improved Somali error messages including line and position information [Mr Sharafdin]

* **feat:** Add madoor keyword for constants with type validation [Mr Sharafdin]

* **feat:** Implement shaandhee (list_filter) method in builtins and update test examples [Mr Sharafdin]

* **feat:** Add shaandhee method for filtering lists in Soplang [Mr Sharafdin]

* **feat:** Allow return statements with and without parentheses for comparison expressions [Mr Sharafdin]

* **feat:** add `habee()` method to sort lists in-place [Mr Sharafdin]

* **feat:** add `rog()` to reverse lists in-place [Mr Sharafdin]

* **feat:** add `nadiifi()` to clear all items from a list [Mr Sharafdin]

* **feat:** add `nuqul()` method for shallow copying lists [Mr Sharafdin]

* **feat:** add `dooro (x)` / `xaalad` switch-case syntax [Mr Sharafdin]

* **feat:** Rename qor to bandhig across the codebase [Elkenzi]

* **feat:** Implement Docker automation for builds and releases [Mr Sharafdin]

* **feat:** enhance changelog generation with GitHub links and improve release process [Mr Sharafdin]

* **feat:** Add error massages [Mr Sharafdin]

### Fixed

* **fix:** rename kudhow to xul in navbar [Muna Abdullahi]

* **fix:** improve handling of comparison operators in expressions [Mr Sharafdin]

* **fix:** update Docker Hub description workflow with improved checks [Mr Sharafdin]

* **fix:** improve Docker Hub description handling and add verification steps [Mr Sharafdin]

* **fix:** Docker build process and workflow authentication [Mr Sharafdin]

* **fix:** correct CHANGELOG.md formatting with proper GitHub links [Mr Sharafdin]

* **fix:** resolve duplicate Unreleased sections and improve username linking in changelog [Mr Sharafdin]

* **fix:** correct changelog format with proper username links and remove duplicate unreleased sections [Mr Sharafdin]

* **fix:** improve changelog generation for first release [Mr Sharafdin]

* **fix:** replace PR approach with direct commits in release workflow [Mr Sharafdin]

* **fix:** resolve GitHub Actions permission issues by using PR approach [Mr Sharafdin]

* **fix:** update release workflow to handle readline dependency issue [Mr Sharafdin]

* **fix:** Update error message formatting to properly display line and position with translations [Mr Sharafdin]

* **fix:** Add support for method calls in parser and interpreter [Mr Sharafdin]

  Fixed an issue in the parser that was failing to handle method calls properly, causing examples like 03_functions.so to fail with the error 'Unexpected token: TokenType.LEFT_PAREN'.

### Changed

* **refactor:** centralize version management and fix Windows version format [Mr Sharafdin]

* **refactor:** centralize version management and fix Windows version format [Mr Sharafdin]

* **refactor:** clarify number types with `tiro` and `jajab` [Mr Sharafdin]

* **refactor:** replace `by` with `::` in loop step syntax [Mr Sharafdin]

* **refactor:** remove  and apply new parentheses-based loop syntax [Mr Sharafdin]

### Documentation

* **docs:** update grammar.ebnf to include daji and built-in functions [Mr Sharafdin]

* **docs:** update documentation for walax.nuqul() method [Mr Sharafdin]

* **docs:** update old Soplang keywords in examples and docs only [iftiiin]

* **docs:** fix grammar inconsistencies in loop syntax and data types [Mr Sharafdin]

* **docs:** update documentation to maintain keyword consistency [Mr Sharafdin]

* **docs:** update documentation for improved operator expression handling and switch-case syntax [Mr Sharafdin]

### Build

* **ci:** update release workflow for manual binary builds [Mr Sharafdin]

* **ci:** add automated release workflow and documentation [Mr Sharafdin]

### Other

* **chore(release):** bump version to 2.0.0-beta for Windows beta release [Mr Sharafdin]

* rm mvp [Mr Sharafdin]

* Fix example files to work with renamed keywords and add test script. [Mr Sharafdin]

* **Rename tiro → abn (scalar:** abyoone) [Abdiladiif-Abdisamed]

* Rename liis → teed [Abdiladiif-Abdisamed]

* Rename raadso() → muuji() [Abdiladiif-Abdisamed]

* Rename-bandhig-to-qor [Abdiladiif-Abdisamed]

* Rename tirtir() → tir() [Abdiladiif-Abdisamed]

* changed delim to xad [Mr Sharafdin]

* **chore:** removed grammar.md file and expressions.md [Mr Sharafdin]

* **chore:** remove object-oriented, error handling, and module import keywords [Mr Sharafdin]

* **Update documentation for Soplang:** grammar.md, keywords.md, and grammar.ebnf with new keywords and methods. [Mr Sharafdin]

* **chore:** remove old examples/errors test folder [iftiiin]

* **Fix number type handling:** treat floats (even with zero decimals) as jajab type and preserve decimal display [Mr Sharafdin]

* Fix kudar method to work with both single values and arrays. Modified list_concat to handle push operations when given a non-list value and concatenate operations when given a list value. [Mr Sharafdin]

* fix comparison operators [Mr Sharafdin]

* fix logical operator [Mr Sharafdin]

* fix the greater and comparison [Mr Sharafdin]

* changed length to dherer [Mr Sharafdin]

* Fix token type inconsistencies in lexer and parser for version 2.0 keywords [Mr Sharafdin]

* Fix token type inconsistencies in lexer and parser for version 2.0 keywords [Mr Sharafdin]

* Update examples directory to reflect changes in keywords for version 2.0 [Mr Sharafdin]

* 🔁 Rename inta_ay to intay in examples [shiinedev]

* 🔁 Rename inta_ay to intay in grammer.ebng [shiinedev]

* 🔁 Rename inta_ay to intay in docs [shiinedev]

* 🔁 Rename inta_ay to intay in src [shiinedev]

* 🔁 Rename nuuc to nooc in errors.py file [shiinedev]

* 🔁 Rename nuuc to nooc in bultins.py  file [shiinedev]

* 🔁 Rename nuuc to nooc in interprete.py  file [shiinedev]

* 🔁Rename nuuc to nooc in user_input.sop  file [shiinedev]

* 🔁Rename nuuc to nooc in type_conversion.sop  file [shiinedev]

* 🔁Rename nuuc to nooc in object_operation.sop  file [shiinedev]

* 🔁Rename nuuc to nooc in type_checking_sop  file [shiinedev]

* 🔁Rename nuuc to nooc in keyword.md  file [shiinedev]

* 🔁Rename nuuc to nooc in errors.c file [shiinedev]

* 🔁Rename nuuc to nooc in builtin.c file [shiinedev]

* 🔁Rename nuuc to nooc in testing [shiinedev]

* 🔁Rename nuuc to nooc in TEST_EXAMPLES_README  file [shiinedev]

* 🔁Rename nuuc to nooc in examples.md file [shiinedev]

* Rename iskuxir to kudar in soplang [Aisha (sarajohn786)]

* Rename iskuxir to kudar in soplang [Aisha (sarajohn786)]

* Rename iskuxir to kudar in docs and grammer [Aisha (sarajohn786)]

* renamed akhri to gelin in examples [Nasrah-muse]

* renamed akhri to gelin in docs [Nasrah-muse]

* renamed akhri to gelin in src [Nasrah-muse]

* renamed shey to walax [star-Elmi]

* renamed shey to walax [star-Elmi]

* renamed howl to hawl [star-Elmi]

* Rename iskudar to kudar in soplang [Aisha (sarajohn786)]

* Rename iskudar to kudar in docs and grammar [Aisha (sarajohn786)]

* **Update:** renamed 'ka_kooban' to 'leeyahay' in grammar.ebnf [juweyriyo]

* **Update:** rename 'ka_kooban' to 'leeyahay' in grammar.md [juweyriyo]

* **Update:** rename 'ka_kooban' to 'leeyahay' in keywords.md [juweyriyo]

* **Update:** rename 'ka_kooban' to 'leeyahay' in EXAMPLES.md [juweyriyo]

* **Update:** rename 'ka_kooban' to 'leeyahay' in 11_list_operations.sop [juweyriyo]

* Rename 'ka_kooban' to 'leeyahay' in builtins.py [juweyriyo]

* **Update grammar definition:** rename 'haystaa' to 'leeyahay' for consistency [juweyriyo]

* **Update grammar and keywords docs:** rename 'haystaa' to 'leeyahay' [juweyriyo]

* **Update examples:** rename 'haystaa' to 'leeyahay' in tests and operations [juweyriyo]

* Rename 'haystaa' to 'leeyahay' in builtins.py [juweyriyo]

* **Update grammar rules:** rename 'furaha' to 'fure' in grammar definitions [juweyriyo]

* **Update grammar doc:** rename 'furaha' to 'fure' for consistency [juweyriyo]

* **Update docs:** rename 'furaha' to 'fure' in keywords and examples [juweyriyo]

* **Update examples:** rename 'furaha' to 'fure' in object operations [juweyriyo]

* Rename 'furaha' to 'fure' in builtins.py [juweyriyo]

* Rename ku_celi to kuceli [Abdiladiif-Abdisamed]

* Rename sii_wad to soco [Abdiladiif-Abdisamed]

* Rename waxba to maran [Abdiladiif-Abdisamed]

* renamed labadaran to bool [star-Elmi]

* renamed haddii_kalena to ugudambeyn in grammar.ebnf [Nasrah-muse]

* renamed haddii_kalena to ugudambeyn in README file [Nasrah-muse]

* renamed haddii_kalena to ugudambeyn in examples [Nasrah-muse]

* renamed haddii_kalena to ugudambeyn in docs [Nasrah-muse]

* rename haddii_kalena to ugudambeyn in tests [Nasrah-muse]

* rename haddii_kalena to ugudambeyn in src [Nasrah-muse]

* renamed howl to hawl [Mr Sharafdin]

* renamed soo_celi to celi [star-Elmi]

* renamed howl to hawl at files test , parser [star-Elmi]

* Renamed howl to hawl [star-Elmi]

* Fix Docker file access issue and add helper script [Mr Sharafdin]

* Update Docker workflow to only run on tags and releases [Mr Sharafdin]

* Simplify Soplang output by removing all decorative elements [Mr Sharafdin]

* Remove Docker Hub description update steps from workflow [Mr Sharafdin]

* Change primary file extension from .so to .sop while keeping .so as secondary [Mr Sharafdin]

* added soplang v1.0.0 changelog [Mr Sharafdin]

* removed unreleased section [Mr Sharafdin]

* changed GITHUB_TOKEN to Soplang_Release_Token [Mr Sharafdin]

* Fix failing tests to match actual implementation behavior [omartood]

* Add comprehensive installation guide and simplify README [Mr Sharafdin]

* Remove legacy scripts from the root directory [Mr Sharafdin]

* Reorganize build system with platform-specific scripts and move legacy scripts [Mr Sharafdin]

* Add macOS build support with app bundle and DMG creation [Mr Sharafdin]

* Add Linux build support with deb package creation [Mr Sharafdin]

* Updated file associations to use main soplang.exe directly instead of batch wrapper [Mr Sharafdin]

* Fix Soplang Command Prompt shortcut to use direct path setting instead of batch script [Mr Sharafdin]

* Fix Soplang launcher scripts to properly run interactive shell [Mr Sharafdin]

* **Fix Windows display and launch issues:** Show Soplang Interpreter in Control Panel and prevent application from closing immediately [Mr Sharafdin]

* Fix soplang verison name [Mr Sharafdin]

* **Improve shell display and icon handling:** Fix formatting issues in Windows console, enhance icon generation, and improve welcome message [Mr Sharafdin]

* **Fix Windows build issues:** add Windows-specific requirements file, improve error handling and permissions in build scripts [Mr Sharafdin]

* Update publisher name to 'Soplang Software Foundation' and simplify version display [Mr Sharafdin]

* Fix file extension, publisher unknow, and readline error [Mr Sharafdin]

* **Fix Windows build issues:** make readline optional, add prompt_toolkit submodules, fix Inno Setup flags [Mr Sharafdin]

* Add Windows build support [Mr Sharafdin]

* fix errors in repl [Mr Sharafdin]

* **Update Soplang grammar and documentation:** add list/object methods, and support Somali boolean literals [Mr Sharafdin]

* Update grammar and documentation with error message format and translation terms [Mr Sharafdin]

* **Fix error examples:** Remove semicolon from runtime_errors.so and update type_errors.so [Mr Sharafdin]

* Refactor parse_statement to include line and position info in generated AST nodes [Mr Sharafdin]

* Add helper method to create AST nodes with line and position info [Mr Sharafdin]

* Add line and position tracking to ASTNode and update interpreter to include location information in error messages [Mr Sharafdin]

* Fix error formatting to properly display Somali error messages with all placeholders replaced [Mr Sharafdin]

* Fix error handling to properly display Somali error messages and make qor() function handle missing arguments [Mr Sharafdin]

* Fix parser errors and update lexer and parser implementation [Mr Sharafdin]

* Simplify error messages to be Somali-only without English translations [Mr Sharafdin]

* **Fix:** Use consistent Somali type names (qoraal, tiro, liis, shey) instead of English type names in error messages and outputs [Mr Sharafdin]

* Update testing documentation with new files and remove outdated ones [Mr Sharafdin]

* **Fix:** Added support for akhri function in expressions and created example 15 for user input [Mr Sharafdin]

* Add support for list and object operations (examples 11 and 12) [Mr Sharafdin]

* fix haddii_kalena [Mr Sharafdin]

* fix haddii_kalena [Mr Sharafdin]

* Fix formatting in the loop step (by) implementation [Mr Sharafdin]

* fix haddii_kalena [Mr Sharafdin]

* Fix logical operators (NOT) with unary operation implementation [Mr Sharafdin]

* Fix comparison operators in assignments and expressions [Mr Sharafdin]

* **Fix:** Added support for unary operators in parser to handle negative numbers [Mr Sharafdin]

* **Fix:** Updated nuuc function to correctly identify boolean types [Mr Sharafdin]

* **Fix:** Changed boolean literals from true/false to been/run and removed unused import [Mr Sharafdin]

* **Reorganized examples:** Split implemented features into individual test files [Mr Sharafdin]

* Update Makefile test target to use new test runner [Omar-Tood]

* **Restructure test runners:** Move to tests/runners directory [Omar-Tood]

* Add main test runner script for all unit tests [Omar-Tood]

* Add interpreter unit tests with test runner script [Omar-Tood]

* Add parser unit tests with test runner script [Omar-Tood]

* Add lexer unit tests with test runner script [Omar-Tood]

* added security link in soplang.org [Mr Sharafdin]

* changed null to waxba [Mr Sharafdin]

* Add Docker support with minimalist image and helper script [Mr Sharafdin]

* Update documentation to reference Makefile and fix Makefile targets [Mr Sharafdin]

* Add Makefile with development commands [Mr Sharafdin]

* **Update CONTRIBUTING.md:** Remove project structure section and add Code of Conduct link [Mr Sharafdin]

* Add pre-commit configuration and update documentation [Mr Sharafdin]

* Add Code of Conduct and update README with CoC reference [Mr Sharafdin]

* Reorganize C source code with modular structure [Mr Sharafdin]

* Restructure codebase with modular organization [Mr Sharafdin]

* Update camelCase keywords to snake_case [Mr Sharafdin]

* Remove camelCase keyword alternatives from lexer and update grammar formatting [Mr Sharafdin]

* Update remaining example files to use consistent snake_case syntax [Mr Sharafdin]

* Standardize Soplang keywords to snake_case format [Mr Sharafdin]

* Update README with concise, professional format [Mr Sharafdin]

* Add development configuration files [Mr Sharafdin]

* Standardize naming convention for shell scripts [Mr Sharafdin]

* Update script path in compare_all_implementations.sh [Mr Sharafdin]

* Update script paths in documentation files [Mr Sharafdin]

* Reorganize script files into logical directories [Mr Sharafdin]

* Update README with Docker setup instructions [Mr Sharafdin]

* Add Docker configuration for development [Mr Sharafdin]

* Restructure documentation to docs/ directory [Mr Sharafdin]

* Fixed bug in parser where files without trailing newlines caused parse errors [Mr Sharafdin]

* Updated keywords.md with correct Soplang keywords to match implementation [Mr Sharafdin]

* added js README.md [Mr Sharafdin]

* Added Nuitka build system for Windows and Unix platforms [Mr Sharafdin]

* Updated soplang_shell.sh with command line flags support [Mr Sharafdin]

* added flags support [Mr Sharafdin]

* Added interactive Soplang shell with keyboard shortcut support [Mr Sharafdin]

* Moved main.py to src directory and created a wrapper script [Mr Sharafdin]

* Update ast.c [Mr Sharafdin]

* added hello.c [Mr Sharafdin]

* added data types [Mr Sharafdin]

* Add test documentation, and edge case examples [Mr Sharafdin]

* added test for unit testing [Mr Sharafdin]

* Add .gitignore file to exclude compiled and temporary files [Mr Sharafdin]

* Add and update C source files for improved implementation [Mr Sharafdin]

* Add comprehensive performance testing tools and documentation [Mr Sharafdin]

* migrating soplang interpreter to c [Mr Sharafdin]

* Updated documentation to consistently use soo_celi for return statements [Mr Sharafdin]

* **Added comprehensive documentation:** README, grammar specification, and keywords reference [Mr Sharafdin]

* Added support for return statements (sooCeli) in parser and interpreter [Mr Sharafdin]

* Standardized loop keywords to snake_case (ku_celi and inta_ay) and added support for logical operators (&&, ||) [Mr Sharafdin]

* Fixed escape sequence issue in basic example file [Mr Sharafdin]

* Fixed bug in parser to support modulo operator in expressions [Mr Sharafdin]

* Updated source files for better code organization [Mr Sharafdin]

* Removed unused files and reorganized codebase structure [Mr Sharafdin]

* Added modulo (%) operator support [Mr Sharafdin]

* Added method call support for lists and objects in interpreter.py [Mr Sharafdin]

* Added list and object methods to builtins.py for Soplang [Mr Sharafdin]

* Added full support for lists (liis) and objects (shey) in Soplang [Mr Sharafdin]

* Added automated test example. [Mr Sharafdin]

* Added script to run all Soplang examples [Mr Sharafdin]

* Updated main.py to run Soplang example files directly [Mr Sharafdin]

* Updated README with comprehensive documentation including static typing [Mr Sharafdin]

* Added user input example with akhri() function [Mr Sharafdin]

* Added example files showcasing Soplang language features [Mr Sharafdin]

* Added test cases for Soplang language features [Mr Sharafdin]

* Added interpreter with static typing and runtime type enforcement [Mr Sharafdin]

* 📌 Added support for comments (// single-line and /* multi-line */) [Mr Sharafdin]

* 📌 Updated AST to support static typing with var_type property [Mr Sharafdin]

* 📌 Updated parser.py to support static typing (tiro, qoraal, labadaran, etc.) [Mr Sharafdin]

* 📌 Updated main.py with src imports and static typing examples [Mr Sharafdin]

* 📌 Updated lexer to support static typing (tiro, qoraal, labadaran, liis, shey) [Mr Sharafdin]

* 🐞 Fixed builtins.py to use 'labadaran' instead of 'boole' for boolean type [Mr Sharafdin]

* 📌 Added static typing tokens (tiro, qoraal, labadaran, liis, shey) [Mr Sharafdin]

* 📌 Added built-in functions with type checking and conversion [Mr Sharafdin]

* 📌 Added custom Somali error messages for Soplang [Mr Sharafdin]

* Added github templates and security.md [sharafdin]

* **Fix file extension:** changed 'sp' to 'so' [Adam-Elmi]

* Rename another.sp to another.so [Mr Sharafdin]

* rebuild soplang with new syntax [Mr Sharafdin]



## [v1.0.0] - 2023-10-23

### Added
- Added full README documentation.
- Added Soplang language sample.
- Added Soplang grammar definition.
- Added MIT License.
- Added Soplang language core files.

### Improved
- Improved and added `return` statement support.

### Completed
- Completed the initial interpreter for Soplang.
