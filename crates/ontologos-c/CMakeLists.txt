cmake_minimum_required(VERSION 3.16)
project(ontologos_c LANGUAGES C CXX)

set(REPO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../.." CACHE PATH "OntoLogos repository root")

if(APPLE)
    set(ONTOLOGOS_LIB_FILE "libontologos_c.dylib")
    set(ONTOLOGOS_LINK_NAME "ontologos_c")
elseif(WIN32)
    set(ONTOLOGOS_LIB_FILE "ontologos_c.dll")
    set(ONTOLOGOS_LINK_NAME "ontologos_c")
else()
    set(ONTOLOGOS_LIB_FILE "libontologos_c.so")
    set(ONTOLOGOS_LINK_NAME "ontologos_c")
endif()

set(ONTOLOGOS_LIB_PATH "${REPO_ROOT}/target/release/${ONTOLOGOS_LIB_FILE}")

add_custom_command(
    OUTPUT "${ONTOLOGOS_LIB_PATH}"
    COMMAND cargo build -p ontologos-c --release --locked
    WORKING_DIRECTORY "${REPO_ROOT}"
    COMMENT "Building libontologos_c"
    VERBATIM
)

add_custom_target(ontologos_c_native DEPENDS "${ONTOLOGOS_LIB_PATH}")

function(configure_smoke target source)
    add_executable(${target} "${source}")
    target_include_directories(${target} PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/include")
    target_link_directories(${target} PRIVATE "${REPO_ROOT}/target/release")
    target_link_libraries(${target} PRIVATE ${ONTOLOGOS_LINK_NAME})
    add_dependencies(${target} ontologos_c_native)
    if(APPLE)
        set_target_properties(${target} PROPERTIES BUILD_RPATH "${REPO_ROOT}/target/release")
    endif()
endfunction()

configure_smoke(smoke_c tests/smoke.c)
configure_smoke(smoke_cpp tests/smoke.cpp)

enable_testing()
add_test(NAME smoke_c COMMAND smoke_c)
add_test(NAME smoke_cpp COMMAND smoke_cpp)
