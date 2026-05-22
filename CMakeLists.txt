cmake_minimum_required(VERSION 3.16)
project(Toole C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Hello le BOP, le backend Windows est bloque en attendant la synchro d'API.
# Revenir sur ce blocage une fois que src/platform/windows/*.c auront les memes
# signatures que les headers dans includes/ (voir docs/architecture.md).
if(WIN32)
  message(WARNING "Backend Windows desactive : API non synchronisee. Seule l'UI Python tournera sans transfert de fichiers.")
  return()
endif()

set(TOOLE_PLATFORM_SOURCES
  src/platform/linux/discovery.c
  src/platform/linux/network.c
  src/platform/linux/file_transfert.c
  src/platform/linux/server_runtime.c
)

add_library(toole_platform STATIC ${TOOLE_PLATFORM_SOURCES})
target_include_directories(toole_platform PUBLIC includes)
target_link_libraries(toole_platform PUBLIC pthread)

add_library(toole_core STATIC src/core/app.c)
target_include_directories(toole_core PUBLIC includes)
target_link_libraries(toole_core PUBLIC toole_platform)

add_library(toole_bridge SHARED src/core/bridge.c)
target_include_directories(toole_bridge PUBLIC includes)
target_link_libraries(toole_bridge PUBLIC toole_core)
