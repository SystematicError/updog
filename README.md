<h1 align="center">Updog</h1>

![Engine analysis showcase](https://github.com/user-attachments/assets/dead6d99-2789-47c4-93d4-ac347d8d8e8d)

Updog is an experimental UCI chess engine. You can interact with it via UCI compatible GUIs like [En Croissant](https://encroissant.org/). 

# Builds

Currently there are no prebuilt binaries. You can build the project yourself using the `make native`, which produces a [pgo](https://doc.rust-lang.org/rustc/profile-guided-optimization.html) optimised build for the current host architecture.

Make, Rust (with `llvm-tools-preview` extension), and `cargo-pgo` are required to build this project.

# Testing

All patches that affect the engine's gameplay are tested using [Sequential Probability Ratio Tests](https://chessprogramming.org/Sequential_Probability_Ratio_Test) to ensure the intended change in engine strength before they ever get merged to `master`. Currently, [OpenBench](https://github.com/SystematicError/updog-bench/) is used to distribute test workloads across multiple machines.

Historical SPRT results can be found [here](https://github.com/SystematicError/updog/pulls?q=is%3Apr+label%3Atests).
