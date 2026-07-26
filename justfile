set lazy
set unstable

executable_name := "updog"
threads := "14"

book := shell(f"echo `nix build --no-link --print-out-paths '{{justfile_dir()}}'#book`/*")
build_bench_executable(branch) := shell(f"nix build --no-link --print-out-paths 'git+file:{{justfile_dir()}}/?ref={{branch}}'") + f"/bin/{{executable_name}}"
branch_picker := `git branch --format='%(refname:short)' | gum choose`

sprt base_branch exp_branch engine_options sprt_options:
    #!/usr/bin/env bash
    set -euxo pipefail

    base_executable="{{build_bench_executable(base_branch)}}"
    exp_executable="{{build_bench_executable(exp_branch)}}"

    fastchess \
        -engine cmd="$exp_executable" name="{{exp_branch}} (exp)" \
        -engine cmd="$base_executable" name="{{base_branch}} (base)" \
        -openings file="{{book}}" format=epd order=random \
        -each {{engine_options}} \
        -sprt {{sprt_options}} \
        -resign movecount=3 score=400 -draw movenumber=40 movecount=8 score=10 \
        -rounds 100000 -concurrency {{threads}} \

    echo "bench" | "$base_executable"
    echo "bench" | "$exp_executable"

sprt-stc:
    just sprt "master" "{{branch_picker}}" "tc=8+0.08" "elo0=0 elo1=3 alpha=0.05 beta=0.05"

sprt-ltc:
    just sprt "master" "{{branch_picker}}" "tc=40+0.4" "elo0=0 elo1=3 alpha=0.05 beta=0.05"

sprt-stc-regression:
    just sprt "master" "{{branch_picker}}" "tc=8+0.08" "elo0=-5 elo1=0 alpha=0.05 beta=0.05"

sprt-ltc-regression:
    just sprt "master" "{{branch_picker}}" "tc=40+0.4" "elo0=-5 elo1=0 alpha=0.05 beta=0.05"
