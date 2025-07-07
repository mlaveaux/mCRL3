add_executable(mcrl3_example
  main.cpp
)

target_link_libraries(mcrl3_example
  PRIVATE
    mcrl3_ffi
)