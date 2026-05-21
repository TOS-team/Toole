cmake_minimum_required(VERSION 3.16)
project(Toole C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Hello le BOP, ici on ne build que le strict necessaire pour la connexion UI
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
