cmake_minimum_required(VERSION 3.16)
project(Toole C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Hello le BOP, ici on genere aussi compile_commands.json pour aider l'editeur
# a retrouver les headers C dans includes/ pendant les diagnostics.
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

if(WIN32)
  # backend Windows
  set(TOOLE_PLATFORM_SOURCES
    src/platform/windows/discovery.c
    src/platform/windows/network.c
    src/platform/windows/file_transfert.c
    src/platform/windows/server_runtime.c
  )
else()
  # backend Linux stable
  set(TOOLE_PLATFORM_SOURCES
    src/platform/linux/discovery.c
    src/platform/linux/network.c
    src/platform/linux/file_transfert.c
    src/platform/linux/server_runtime.c
  )
endif()

# lib plateforme (reseau + runtime)
add_library(toole_platform STATIC ${TOOLE_PLATFORM_SOURCES})
target_include_directories(toole_platform PUBLIC includes)

if(UNIX AND NOT APPLE)
  target_link_libraries(toole_platform PUBLIC pthread)
endif()

if(WIN32)
  target_link_libraries(toole_platform PUBLIC ws2_32)
endif()

# coeur applicatif
add_library(toole_core STATIC src/core/app.c)
target_include_directories(toole_core PUBLIC includes)
target_link_libraries(toole_core PUBLIC toole_platform)

# bridge C pour l'UI (ctypes)
add_library(toole_bridge SHARED src/core/bridge.c)
target_include_directories(toole_bridge PUBLIC includes)
target_link_libraries(toole_bridge PUBLIC toole_core)

# Hello le BOP, les tests actuels valident le socle Linux uniquement.
# On les branche donc au build Linux sans toucher a la partie Windows.
if(UNIX AND NOT APPLE)
  enable_testing()

  add_executable(discovery_test tests/discovery_test.c)
  target_include_directories(discovery_test PRIVATE includes)
  target_link_libraries(discovery_test PRIVATE toole_platform)
  add_test(NAME discovery_test COMMAND discovery_test)

  add_executable(network_test tests/network_test.c)
  target_include_directories(network_test PRIVATE includes)
  target_link_libraries(network_test PRIVATE toole_platform)
  add_test(NAME network_test COMMAND network_test)

  add_executable(file_transfer_test tests/file_transfer_test.c)
  target_include_directories(file_transfer_test PRIVATE includes)
  target_link_libraries(file_transfer_test PRIVATE toole_platform)
  add_test(NAME file_transfer_test COMMAND file_transfer_test)

  add_executable(server_runtime_test tests/server_runtime_test.c)
  target_include_directories(server_runtime_test PRIVATE includes)
  target_link_libraries(server_runtime_test PRIVATE toole_platform)
  add_test(NAME server_runtime_test COMMAND server_runtime_test)

  find_package(Python3 COMPONENTS Interpreter)
  if(Python3_Interpreter_FOUND)
    add_test(
      NAME bridge_ctypes_test
      COMMAND ${Python3_EXECUTABLE} ${CMAKE_SOURCE_DIR}/tests_py/test_bridge_ctypes.py
    )
  endif()
endif()
