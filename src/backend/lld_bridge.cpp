#include "lld/Common/Driver.h"
#include "llvm/Support/raw_ostream.h"

#include <algorithm>
#include <cstring>
#include <string>
#include <vector>

LLD_HAS_DRIVER(elf)

extern "C" int soplang_lld_elf(int argc, const char *const *argv,
                               char *diagnostics, size_t diagnostics_len) {
  static bool reusable = true;
  if (!reusable) {
    const char *message = "LLD cannot safely run again in this process";
    if (diagnostics_len != 0) {
      const auto len = std::min(std::strlen(message), diagnostics_len - 1);
      std::memcpy(diagnostics, message, len);
      diagnostics[len] = '\0';
    }
    return 1;
  }
  std::vector<const char *> args(argv, argv + argc);
  std::string output;
  llvm::raw_string_ostream stream(output);
  const lld::DriverDef drivers[] = {{lld::Gnu, &lld::elf::link}};
  auto result = lld::lldMain(args, stream, stream, drivers);
  reusable = result.canRunAgain;
  stream.flush();
  if (diagnostics_len != 0) {
    const auto len = std::min(output.size(), diagnostics_len - 1);
    std::memcpy(diagnostics, output.data(), len);
    diagnostics[len] = '\0';
  }
  return result.retCode;
}
