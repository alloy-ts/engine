cmake_minimum_required(VERSION 3.18)
project(AlloyScript LANGUAGES C CXX)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

option(ALLOY_ENABLE_WEB "Enable alloy:web proxy process and browser APIs" ON)

include(CheckIPOSupported)
check_ipo_supported(RESULT ipo_supported OUTPUT error)
if(ipo_supported)
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION ON)
endif()

if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU" OR CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -static-libgcc -static-libstdc++")
endif()

find_package(PkgConfig REQUIRED)
pkg_check_modules(SODIUM REQUIRED libsodium)

include_directories(include)
include_directories(packages/mquickjs)

set(CORE_SOURCES
    src/core/engine.c
    src/core/main.c
    src/core/stdlib/custom_stdlib.c
    src/core/plugins/config/config_plugin.c
    src/core/crypto/encrypted_ram.c
    src/core/crypto/crypto.c
    src/core/capability/store.c
    src/core/capability/validator.c
    src/core/ipc/broker.c
    src/core/ipc/channel.c
    src/repl/repl_main.c
    packages/mquickjs/mquickjs.c
    packages/mquickjs/dtoa.c
    packages/mquickjs/libm.c
    packages/mquickjs/cutils.c
    packages/mquickjs/readline.c
)

if(ALLOY_ENABLE_WEB)
    add_definitions(-DALLOY_ENABLE_WEB=1)
    list(APPEND CORE_SOURCES src/core/plugins/web/web_plugin.c src/core/plugins/shell/shell_plugin.c)

    set(WEB_SOURCES src/web/main.c src/web/webview.c src/web/rpc_handler.c)
    add_executable(alloy_web ${WEB_SOURCES})
    if(APPLE)
        find_library(WEBKIT_FRAMEWORK WebKit REQUIRED)
        target_link_libraries(alloy_web PRIVATE ${WEBKIT_FRAMEWORK})
    elseif(WIN32)
    else()
        pkg_check_modules(GTK3 REQUIRED gtk+-3.0)
        pkg_check_modules(WEBKIT2 REQUIRED webkit2gtk-4.1)
        target_include_directories(alloy_web PRIVATE ${GTK3_INCLUDE_DIRS} ${WEBKIT2_INCLUDE_DIRS} packages/webview/core/include)
        target_link_libraries(alloy_web PRIVATE ${GTK3_LIBRARIES} ${WEBKIT2_LIBRARIES})
    endif()
else()
    add_definitions(-DALLOY_ENABLE_WEB=0)
endif()

add_executable(alloy_core ${CORE_SOURCES})
target_include_directories(alloy_core PRIVATE ${SODIUM_INCLUDE_DIRS})
target_link_libraries(alloy_core PRIVATE ${SODIUM_LIBRARIES} m)
