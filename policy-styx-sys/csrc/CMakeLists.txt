# Set the minimum required version of CMake
cmake_minimum_required(VERSION 3.20)

# Set the project name
project(PCDPolicySystem)

set(WASI_TARGET "wasm32-wasi")
set(WASI_SDK "/opt/wasi-sdk")
set(WASI_OPENSSL "/opt/openssl-wasm")

# Set the C and C++ standards
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

# Use Emscripten compilers
set(CMAKE_C_COMPILER "emcc")
set(CMAKE_CXX_COMPILER "em++")

# Add the source directories
set(SRC_DIR ${CMAKE_SOURCE_DIR})

# Include directories
include_directories(${SRC_DIR}/include)
include_directories(${SRC_DIR}/runtime/wamr/wasm-micro-runtime/core/iwasm/include)
include_directories(${WASI_OPENSSL}/include)
# Add source files
file(GLOB_RECURSE FRAMEWORK_SOURCES ${SRC_DIR}/framework/*.c)
file(GLOB_RECURSE CRYPTO_SOURCES ${SRC_DIR}/crypto/*.c)
file(GLOB_RECURSE POLICY_SOURCES ${SRC_DIR}/policy/*.c)

set (CMAKE_CXX_FLAGS "${CMAKE_C_FLAGS} -s STANDALONE_WASM")
set (CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s STANDALONE_WASM")

# Secret.cpp is C++; treat it separately
set(SECRET_CPP ${SRC_DIR}/framework/secret.cpp)

# Combine all source files into one list
set(SOURCES
    ${FRAMEWORK_SOURCES}
    ${CRYPTO_SOURCES}
    ${POLICY_SOURCES}
    ${SECRET_CPP}
)

add_definitions(-DPCD_CONFIG_CRYPTO_AES_GCM)

# Create a static library. 
add_library(pcd STATIC ${SOURCES})

# Set the output library name
set_target_properties(pcd PROPERTIES OUTPUT_NAME "pcd")

# Add flags and link libraries if necessary
# Example: target_compile_options(pcd PRIVATE -Wall -Wextra)
# Example: target_link_libraries(pcd PRIVATE pthread)
install(TARGETS pcd
    LIBRARY DESTINATION lib           # Install shared library in `lib`
    ARCHIVE DESTINATION lib           # For static library (if applicable)
    RUNTIME DESTINATION bin           # For executables (if there are any)
)
