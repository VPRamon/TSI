cmake_minimum_required(VERSION 3.14)
project(stars_ffi VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Find required packages
find_package(PkgConfig REQUIRED)

# Find STARS Core
# Try to find it via FindSTARSCore.cmake (installed by stars-core)
find_package(STARSCore)

if(STARSCore_FOUND)
    message(STATUS "Found STARSCore via CMake config")
    if(NOT TARGET stars::stars-core)
        add_library(stars::stars-core INTERFACE IMPORTED)
        
        # Link against all components found
        set(STARS_COMPONENTS TIME COORDINATES ASTRO SCHEDULER UTILS CONSTRAINTS SCHEDULING_BLOCKS SERIALIZATION JSON_ARCHIVES XML_BUILDER JSON_BUILDER ANALYSIS)
        foreach(COMP ${STARS_COMPONENTS})
             string(TOLOWER ${COMP} LOWER_COMP)
             if(TARGET STARS::Core::${LOWER_COMP})
                 target_link_libraries(stars::stars-core INTERFACE STARS::Core::${LOWER_COMP})
             endif()
        endforeach()
        
        if(STARSCore_INCLUDE_DIRS)
            target_include_directories(stars::stars-core INTERFACE "${STARSCore_INCLUDE_DIRS}")
        elseif(STARSCore_INCLUDE_DIR)
            target_include_directories(stars::stars-core INTERFACE "${STARSCore_INCLUDE_DIR}")
        endif()
    endif()
    set(STARS_CORE_FOUND TRUE)
else()
    message(WARNING "find_package(STARSCore) failed. Attempting manual discovery of components.")
    
    # Check for include dir
    find_path(STARS_CORE_INCLUDE_DIR 
        NAMES stars/time/time_utc.h
        HINTS /usr/local/include /usr/include
    )
    
    if(STARS_CORE_INCLUDE_DIR)
        message(STATUS "Found include dir manually: ${STARS_CORE_INCLUDE_DIR}")
        
        # Create the interface library
        if(NOT TARGET stars::stars-core)
            add_library(stars::stars-core INTERFACE IMPORTED)
            target_include_directories(stars::stars-core INTERFACE "${STARS_CORE_INCLUDE_DIR}")
        endif()
        
        # Find all component libraries manually
        # Note: Library names are like libstars_core_time.so
        set(STARS_COMPONENTS TIME COORDINATES ASTRO SCHEDULER UTILS CONSTRAINTS SCHEDULING_BLOCKS SERIALIZATION JSON_ARCHIVES XML_BUILDER JSON_BUILDER ANALYSIS)
        
        foreach(COMP ${STARS_COMPONENTS})
             string(TOLOWER ${COMP} LOWER_COMP)
             find_library(STARS_LIB_${COMP} 
                 NAMES stars_core_${LOWER_COMP} 
                 HINTS /usr/local/lib /usr/lib
             )
             if(STARS_LIB_${COMP})
                 message(STATUS "Found component ${COMP}: ${STARS_LIB_${COMP}}")
                 target_link_libraries(stars::stars-core INTERFACE "${STARS_LIB_${COMP}}")
             else()
                 message(WARNING "Could not find component library for ${COMP}")
             endif()
        endforeach()
        
        set(STARS_CORE_FOUND TRUE)
    else()
        message(FATAL_ERROR "Could not manually locate stars-core headers (stars/time/time_utc.h) in /usr/local/include or /usr/include")
    endif()
endif()

# Find additional dependencies that STARS Core needs
find_package(Boost 1.74.0 REQUIRED COMPONENTS 
    regex system filesystem date_time thread log log_setup serialization)
find_package(Nova QUIET)
if(NOT Nova_FOUND)
    # Nova library - fallback to manual find
    find_library(NOVA_LIBRARY NAMES nova HINTS /usr/local/lib /usr/lib)
    find_path(NOVA_INCLUDE_DIR NAMES libnova/libnova.h HINTS /usr/local/include /usr/include)
endif()

# Find nlohmann_json
find_package(nlohmann_json 3.11 QUIET)
if(NOT nlohmann_json_FOUND)
    pkg_check_modules(NLOHMANN_JSON REQUIRED nlohmann_json)
    add_library(nlohmann_json::nlohmann_json INTERFACE IMPORTED)
    set_target_properties(nlohmann_json::nlohmann_json PROPERTIES
        INTERFACE_INCLUDE_DIRECTORIES "${NLOHMANN_JSON_INCLUDE_DIRS}"
    )
endif()

# Build options
option(STARS_FFI_BUILD_STATIC "Build static library" OFF)
option(STARS_FFI_BUILD_SHARED "Build shared library" ON)

# Define library sources
set(STARS_FFI_SOURCES
    src/stars_ffi.cpp
)

set(STARS_FFI_HEADERS
    include/stars_ffi.h
)

# Create shared library
if(STARS_FFI_BUILD_SHARED)
    add_library(stars_ffi SHARED ${STARS_FFI_SOURCES})
    target_include_directories(stars_ffi
        PUBLIC
            $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
            $<INSTALL_INTERFACE:include>
    )
    target_link_libraries(stars_ffi
        PRIVATE
            stars::stars-core
            nlohmann_json::nlohmann_json
            Boost::regex
            Boost::system
            Boost::filesystem
            Boost::date_time
            Boost::thread
            Boost::log
            Boost::log_setup
            Boost::serialization
    )
    if(NOVA_LIBRARY)
        target_link_libraries(stars_ffi PRIVATE ${NOVA_LIBRARY})
    endif()
    set_target_properties(stars_ffi PROPERTIES
        VERSION ${PROJECT_VERSION}
        SOVERSION 0
        PUBLIC_HEADER "${STARS_FFI_HEADERS}"
    )
endif()

# Create static library
if(STARS_FFI_BUILD_STATIC)
    add_library(stars_ffi_static STATIC ${STARS_FFI_SOURCES})
    target_include_directories(stars_ffi_static
        PUBLIC
            $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
            $<INSTALL_INTERFACE:include>
    )
    target_link_libraries(stars_ffi_static
        PRIVATE
            stars::stars-core
            nlohmann_json::nlohmann_json
            Boost::regex
            Boost::system
            Boost::filesystem
            Boost::date_time
            Boost::thread
            Boost::log
            Boost::log_setup
            Boost::serialization
    )
    if(NOVA_LIBRARY)
        target_link_libraries(stars_ffi_static PRIVATE ${NOVA_LIBRARY})
    endif()
    set_target_properties(stars_ffi_static PROPERTIES
        OUTPUT_NAME stars_ffi
    )
endif()

# Installation
include(GNUInstallDirs)

if(STARS_FFI_BUILD_SHARED)
    install(TARGETS stars_ffi
        EXPORT stars_ffiTargets
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
        RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
        PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
    )
endif()

if(STARS_FFI_BUILD_STATIC)
    install(TARGETS stars_ffi_static
        EXPORT stars_ffiTargets
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
    )
endif()

# pkg-config file
configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/stars_ffi.pc.in
    ${CMAKE_CURRENT_BINARY_DIR}/stars_ffi.pc
    @ONLY
)
install(FILES ${CMAKE_CURRENT_BINARY_DIR}/stars_ffi.pc
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/pkgconfig
)

# CMake config files
install(EXPORT stars_ffiTargets
    FILE stars_ffiTargets.cmake
    NAMESPACE stars::
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/stars_ffi
)

include(CMakePackageConfigHelpers)
write_basic_package_version_file(
    ${CMAKE_CURRENT_BINARY_DIR}/stars_ffiConfigVersion.cmake
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY SameMajorVersion
)

configure_package_config_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/cmake/stars_ffiConfig.cmake.in
    ${CMAKE_CURRENT_BINARY_DIR}/stars_ffiConfig.cmake
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/stars_ffi
)

install(FILES
    ${CMAKE_CURRENT_BINARY_DIR}/stars_ffiConfig.cmake
    ${CMAKE_CURRENT_BINARY_DIR}/stars_ffiConfigVersion.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/stars_ffi
)
