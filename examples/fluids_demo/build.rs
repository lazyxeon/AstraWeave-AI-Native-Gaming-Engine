// Increase stack size on Windows for release builds
// The State struct is large enough to cause stack overflow in release mode

fn main() {
    // On Windows with MSVC, increase stack size to 8MB
    #[cfg(all(windows, target_env = "msvc"))]
    {
        println!("cargo:rustc-link-arg=/STACK:8388608");
    }

    // On Windows with GNU, use different flag
    #[cfg(all(windows, target_env = "gnu"))]
    {
        println!("cargo:rustc-link-arg=-Wl,--stack,8388608");
    }
}
