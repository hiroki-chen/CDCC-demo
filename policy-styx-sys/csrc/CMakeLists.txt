# Set the minimum required version of CMake
cmake_minimum_required(VERSION 3.20)

# Set the project name
project(PCDPolicySystem)

# Set the C and C++ standards
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

# Add the source directories
set(SRC_DIR ${CMAKE_SOURCE_DIR})

# Include directories
include_directories(${SRC_DIR}/include)
include_directories(${SRC_DIR}/runtime/wamr/wasm-micro-runtime/core/iwasm/include)
# Add source files
file(GLOB_RECURSE FRAMEWORK_SOURCES ${SRC_DIR}/framework/*.c)
file(GLOB_RECURSE CRYPTO_SOURCES ${SRC_DIR}/crypto/*.c)
file(GLOB_RECURSE POLICY_SOURCES ${SRC_DIR}/policy/*.c)

# Secret.cpp is C++; treat it separately
set(SECRET_CPP ${SRC_DIR}/framework/secret.cpp)

# Combine all source files into one list
set(SOURCES
    ${FRAMEWORK_SOURCES}
    ${CRYPTO_SOURCES}
    ${POLICY_SOURCES}
    ${SECRET_CPP}
)

# Create a shared library (dynamic library)
add_library(pcd SHARED ${SOURCES})

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
