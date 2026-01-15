cmake_minimum_required(VERSION 3.14)

project(stars_ffi VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# ============================================================================
# Options
# ============================================================================

option(STARS_FFI_BUILD_STATIC "Build static library instead of shared" OFF)
option(STARS_FFI_INSTALL_HEADERS "Install header files" ON)

# ============================================================================
# Find STARS Core
# ============================================================================

# Try to find STARS Core via CMake config first
find_package(STARSCore QUIET)

if(NOT STARSCore_FOUND)
    # Fallback: look for STARS Core in common locations
    set(STARS_CORE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../core" CACHE PATH "Path to STARS Core source")
    
    if(EXISTS "${STARS_CORE_ROOT}/CMakeLists.txt")
        message(STATUS "Found STARS Core at: ${STARS_CORE_ROOT}")
        
        # Add STARS Core as subdirectory if building from source
        option(STARS_FFI_BUILD_CORE "Build STARS Core from source" ON)
        
        if(STARS_FFI_BUILD_CORE)
            set(BUILD_TESTS OFF CACHE BOOL "" FORCE)
            set(BUILD_DOCUMENTATION OFF CACHE BOOL "" FORCE)
            add_subdirectory(${STARS_CORE_ROOT} ${CMAKE_BINARY_DIR}/stars_core)
        endif()
    else()
        message(FATAL_ERROR "STARS Core not found. Set STARS_CORE_ROOT or install stars-core package.")
    endif()
endif()

# ============================================================================
# Find nlohmann_json
# ============================================================================

find_package(nlohmann_json 3.11.0 REQUIRED)

# ============================================================================
# Library Sources
# ============================================================================

set(STARS_FFI_SOURCES
    src/stars_ffi.cpp
)

set(STARS_FFI_HEADERS
    include/stars_ffi.h
)

# ============================================================================
# Library Target
# ============================================================================

if(STARS_FFI_BUILD_STATIC)
    add_library(stars_ffi STATIC ${STARS_FFI_SOURCES} ${STARS_FFI_HEADERS})
else()
    add_library(stars_ffi SHARED ${STARS_FFI_SOURCES} ${STARS_FFI_HEADERS})
endif()

add_library(stars::ffi ALIAS stars_ffi)

target_include_directories(stars_ffi
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
    PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src
)

# Link to STARS Core components
target_link_libraries(stars_ffi
    PRIVATE
        nlohmann_json::nlohmann_json
)

# Link STARS Core libraries based on how they're available
if(TARGET SERIALIZATION)
    target_link_libraries(stars_ffi PRIVATE 
        SERIALIZATION
        SCHEDULING_BLOCKS
        SCHEDULER
        JSON_BUILDER
        CONSTRAINTS
        TIME
        COORDINATES
        ASTRO
    )
elseif(TARGET stars::serialization)
    target_link_libraries(stars_ffi PRIVATE
        stars::serialization
        stars::scheduling_blocks
        stars::scheduler
        stars::json_builder
        stars::constraints
        stars::time
        stars::coordinates
        stars::astro
    )
else()
    # Link by library name (installed case)
    target_link_libraries(stars_ffi PRIVATE
        stars-serialization
        stars-scheduling-blocks
        stars-scheduler
        stars-json-builder
        stars-constraints
        stars-time
        stars-coordinates
        stars-astro
    )
endif()

# Set version properties
set_target_properties(stars_ffi PROPERTIES
    VERSION ${PROJECT_VERSION}
    SOVERSION ${PROJECT_VERSION_MAJOR}
    PUBLIC_HEADER "${STARS_FFI_HEADERS}"
)

# Ensure C symbols are exported correctly
if(NOT STARS_FFI_BUILD_STATIC)
    if(WIN32)
        target_compile_definitions(stars_ffi PRIVATE STARS_FFI_EXPORTS)
    else()
        set_target_properties(stars_ffi PROPERTIES
            C_VISIBILITY_PRESET default
            CXX_VISIBILITY_PRESET default
            VISIBILITY_INLINES_HIDDEN OFF
        )
    endif()
endif()

# ============================================================================
# Installation
# ============================================================================

include(GNUInstallDirs)

install(TARGETS stars_ffi
    EXPORT stars_ffi-targets
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)

install(EXPORT stars_ffi-targets
    FILE stars_ffi-targets.cmake
    NAMESPACE stars::
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/stars_ffi
)

# Generate and install config file
include(CMakePackageConfigHelpers)

configure_package_config_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/cmake/stars_ffi-config.cmake.in
    ${CMAKE_CURRENT_BINARY_DIR}/stars_ffi-config.cmake
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/stars_ffi
)

write_basic_package_version_file(
    ${CMAKE_CURRENT_BINARY_DIR}/stars_ffi-config-version.cmake
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY SameMajorVersion
)

install(FILES
    ${CMAKE_CURRENT_BINARY_DIR}/stars_ffi-config.cmake
    ${CMAKE_CURRENT_BINARY_DIR}/stars_ffi-config-version.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/stars_ffi
)

# ============================================================================
# pkg-config
# ============================================================================

configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/cmake/stars_ffi.pc.in
    ${CMAKE_CURRENT_BINARY_DIR}/stars_ffi.pc
    @ONLY
)

install(FILES
    ${CMAKE_CURRENT_BINARY_DIR}/stars_ffi.pc
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/pkgconfig
)
