
fn main() {
    // Use the `cmake` crate to build cpptrace.
    let mut dst = cmake::Config::new("mt-kahypar/")
        .define("BUILD_SHARED_LIBS", "OFF") // Build a static library.
        .define("KAHYPAR_STATIC_LINK_DEPENDENCIES ", "ON") // Statically link dependencies.
        .define("KAHYPAR_DOWNLOAD_TBB", "ON") // Let cmake download TBB.
        .define("KAHYPAR_DOWNLOAD_BOOST", "ON") // Let cmake download Boost.
        .define("KAHYPAR_DISABLE_HWLOC", "ON") // Disable hwloc to avoid dependency issues.
        .build();
    dst.push("lib");

        cargo_emit::rustc_link_search!(dst.display() => "native");
    // Link the required libraries for cpptrace (Can this be derived from the cmake somehow?)
    cargo_emit::rustc_link_lib!("cpptrace" => "static");
}
