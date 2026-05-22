cmake_minimum_required(VERSION 3.16)
project(TooleTests C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

enable_testing()

set(TOOLE_PLATFORM_SOURCES
  ../src/platform/linux/discovery.c
  ../src/platform/linux/network.c
  ../src/platform/linux/file_transfert.c
  ../src/platform/linux/server_runtime.c
)

add_library(toole_platform_test STATIC ${TOOLE_PLATFORM_SOURCES})
target_include_directories(toole_platform_test PUBLIC ../includes)
target_link_libraries(toole_platform_test PUBLIC pthread)

add_executable(discovery_test discovery_test.c)
target_include_directories(discovery_test PRIVATE ../includes)
target_link_libraries(discovery_test PRIVATE toole_platform_test)
add_test(NAME discovery_test COMMAND discovery_test)

add_executable(network_test network_test.c)
target_include_directories(network_test PRIVATE ../includes)
target_link_libraries(network_test PRIVATE toole_platform_test)
add_test(NAME network_test COMMAND network_test)

add_executable(file_transfer_test file_transfer_test.c)
target_include_directories(file_transfer_test PRIVATE ../includes)
target_link_libraries(file_transfer_test PRIVATE toole_platform_test)
add_test(NAME file_transfer_test COMMAND file_transfer_test)

add_executable(server_runtime_test server_runtime_test.c)
target_include_directories(server_runtime_test PRIVATE ../includes)
target_link_libraries(server_runtime_test PRIVATE toole_platform_test)
add_test(NAME server_runtime_test COMMAND server_runtime_test)
