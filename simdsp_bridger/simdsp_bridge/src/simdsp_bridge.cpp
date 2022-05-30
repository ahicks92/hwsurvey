#include <simdsp/system_info.hpp>

#include <stdlib.h>

extern "C" char *simdspBridgeGetSystemInfoAsJson()
{
    auto info = simdsp::getSystemInfo();
    return simdsp::convertSystemInfoToJson(&info);
}

extern "C" void simdspBridgeFreeJsonString(char *what)
{
    free(what);
}
