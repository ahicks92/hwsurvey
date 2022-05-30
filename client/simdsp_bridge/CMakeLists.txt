cmake_minimum_required(VERSION 3.15.0)
project(simdsp C CXX)

include(FetchContent)

# Apparently we have to do this globally because CMake won't let us set a define for a subdirectory. Without it
# google/benchmark wants gtest.
set(BENCHMARK_ENABLE_TESTING OFF)

FetchContent_Declare(benchmark
  GIT_REPOSITORY https://github.com/google/benchmark
  GIT_TAG 37be1e8252527229cccad9f097afe68572f3c08a
)

FetchContent_Declare(catch2
  GIT_REPOSITORY https://github.com/catchorg/Catch2
  GIT_TAG 62fd660583d3ae7a7886930b413c3c570e89786c
)

FetchContent_Declare(simdsp
  GIT_REPOSITORY https://github.com/synthizer/simdsp
  GIT_TAG fb6276ff5d831746da093a9036f1ad6329018959
)

FetchContent_MakeAvailable(benchmark catch2 simdsp)

add_library(simdsp_bridge STATIC src/simdsp_bridge.cpp)
set_property(TARGET simdsp_bridge PROPERTY CXX_STANDARD 17)
target_link_libraries(simdsp_bridge simdsp)

install(
  TARGETS simdsp_bridge
  LIBRARY DESTINATION  "${CMAKE_INSTALL_LIBDIR}"
  ARCHIVE DESTINATION  "${CMAKE_INSTALL_LIBDIR}"
  RUNTIME DESTINATION  "${CMAKE_INSTALL_BINDIR}"
  INCLUDES DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}"
)
