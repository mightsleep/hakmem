window.BENCHMARK_DATA = {
  "lastUpdate": 1789842421744,
  "repoUrl": "https://github.com/mightsleep/hakmem",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "acannix@proton.me",
            "name": "Michal",
            "username": "mightsleep"
          },
          "committer": {
            "email": "acannix@proton.me",
            "name": "Michal",
            "username": "mightsleep"
          },
          "distinct": true,
          "id": "24d2c0788f59b40211de1cfd58dcb60384fde7bf",
          "message": "hakmem 0.1.0: initial import\n\nCI: crane matrix, nextest, doctests, MSRV from package tarball\n\nCI: dynamic matrix from the flake, treefmt, gh-pages docs and bench trend\n\nCI: cargo-deny and audit, dependabot",
          "timestamp": "2026-09-19T17:48:47+02:00",
          "tree_id": "5238c57f12f51b00ad6d12b33ce340d13faa5b07",
          "url": "https://github.com/mightsleep/hakmem/commit/24d2c0788f59b40211de1cfd58dcb60384fde7bf"
        },
        "date": 1789834799153,
        "tool": "cargo",
        "benches": [
          {
            "name": "select64_vs_broadword/hakmem",
            "value": 682,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/broadword::select1_raw",
            "value": 3092,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m16_n64",
            "value": 269,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m16_n64",
            "value": 448,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m16_n64",
            "value": 1348,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m16_n64",
            "value": 1809,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m16_n64",
            "value": 1441,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m16_n64",
            "value": 1037,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m16_n64",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m16_n64",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m16_n64",
            "value": 1414,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m32_n256",
            "value": 993,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m32_n256",
            "value": 1517,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m32_n256",
            "value": 4454,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m32_n256",
            "value": 6327,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m32_n256",
            "value": 29693,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m32_n256",
            "value": 11533,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m32_n256",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m32_n256",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m32_n256",
            "value": 10518,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n256",
            "value": 1014,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n256",
            "value": 1560,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n256",
            "value": 5042,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n256",
            "value": 6651,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n256",
            "value": 32822,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n256",
            "value": 12827,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m64_n256",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m64_n256",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n256",
            "value": 20943,
            "range": "± 290",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n1024",
            "value": 3879,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n1024",
            "value": 5738,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n1024",
            "value": 15942,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n1024",
            "value": 24047,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n1024",
            "value": 405335,
            "range": "± 882",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n1024",
            "value": 378900,
            "range": "± 7200",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m64_n1024",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m64_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n1024",
            "value": 82232,
            "range": "± 506",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m128_n1024",
            "value": 5821,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m128_n1024",
            "value": 17374,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m128_n1024",
            "value": 24750,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m128_n1024",
            "value": 431568,
            "range": "± 2297",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m128_n1024",
            "value": 405227,
            "range": "± 4228",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m128_n1024",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m128_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m128_n1024",
            "value": 164501,
            "range": "± 571",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m256_n1024",
            "value": 19906,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m256_n1024",
            "value": 25958,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m256_n1024",
            "value": 477077,
            "range": "± 809",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m256_n1024",
            "value": 462889,
            "range": "± 1006",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m256_n1024",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m256_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m256_n1024",
            "value": 329094,
            "range": "± 816",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m512_n1024",
            "value": 28365,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m512_n1024",
            "value": 576631,
            "range": "± 15437",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m512_n1024",
            "value": 543938,
            "range": "± 977",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m512_n1024",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m512_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m512_n1024",
            "value": 657303,
            "range": "± 6943",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u64/len64_edits4",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len64_edits4",
            "value": 507,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len64_edits4",
            "value": 2283,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k8/len64_edits4",
            "value": 1549,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len64_edits4",
            "value": 1548,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len64_edits4",
            "value": 5555,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len128_edits8",
            "value": 935,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len128_edits8",
            "value": 4375,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k16/len128_edits8",
            "value": 2811,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len128_edits8",
            "value": 2806,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len128_edits8",
            "value": 21282,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits8",
            "value": 16595,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k16/len512_edits8",
            "value": 10392,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits8",
            "value": 10390,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits8",
            "value": 330372,
            "range": "± 1141",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits32",
            "value": 16588,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k64/len512_edits32",
            "value": 17711,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits32",
            "value": 10387,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits32",
            "value": 331284,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/dense",
            "value": 681,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/dense",
            "value": 3360,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/dense",
            "value": 5086,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/sparse",
            "value": 682,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/sparse",
            "value": 3362,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/sparse",
            "value": 2558,
            "range": "± 16",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "acannix@proton.me",
            "name": "Michal",
            "username": "mightsleep"
          },
          "committer": {
            "email": "acannix@proton.me",
            "name": "Michal",
            "username": "mightsleep"
          },
          "distinct": true,
          "id": "6c53747a169e20f571f854b655d7eca01bc94879",
          "message": "CI: bump action pins & criterion 0.8, README clippy",
          "timestamp": "2026-09-19T18:24:28+02:00",
          "tree_id": "4426fce59464bec00a372a8d23e280a6173d364f",
          "url": "https://github.com/mightsleep/hakmem/commit/6c53747a169e20f571f854b655d7eca01bc94879"
        },
        "date": 1789835967967,
        "tool": "cargo",
        "benches": [
          {
            "name": "select64_vs_broadword/hakmem",
            "value": 696,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/broadword::select1_raw",
            "value": 3092,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m16_n64",
            "value": 262,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m16_n64",
            "value": 427,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m16_n64",
            "value": 1344,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m16_n64",
            "value": 1817,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m16_n64",
            "value": 1451,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m16_n64",
            "value": 1039,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m16_n64",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m16_n64",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m16_n64",
            "value": 1402,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m32_n256",
            "value": 987,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m32_n256",
            "value": 1415,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m32_n256",
            "value": 4407,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m32_n256",
            "value": 6342,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m32_n256",
            "value": 31556,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m32_n256",
            "value": 11518,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m32_n256",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m32_n256",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m32_n256",
            "value": 10535,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n256",
            "value": 1008,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n256",
            "value": 1471,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n256",
            "value": 5034,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n256",
            "value": 6685,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n256",
            "value": 33996,
            "range": "± 468",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n256",
            "value": 12827,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m64_n256",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m64_n256",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n256",
            "value": 20847,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n1024",
            "value": 3873,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n1024",
            "value": 5384,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n1024",
            "value": 15697,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n1024",
            "value": 24106,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n1024",
            "value": 415392,
            "range": "± 1493",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n1024",
            "value": 393350,
            "range": "± 1557",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m64_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m64_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n1024",
            "value": 82177,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m128_n1024",
            "value": 5451,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m128_n1024",
            "value": 17190,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m128_n1024",
            "value": 24874,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m128_n1024",
            "value": 437021,
            "range": "± 5617",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m128_n1024",
            "value": 418699,
            "range": "± 1271",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m128_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m128_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m128_n1024",
            "value": 164136,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m256_n1024",
            "value": 19636,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m256_n1024",
            "value": 26081,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m256_n1024",
            "value": 484842,
            "range": "± 12075",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m256_n1024",
            "value": 471503,
            "range": "± 1624",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m256_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m256_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m256_n1024",
            "value": 328170,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m512_n1024",
            "value": 28516,
            "range": "± 407",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m512_n1024",
            "value": 585487,
            "range": "± 4620",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m512_n1024",
            "value": 571975,
            "range": "± 2453",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k4/m512_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_simd_k32/m512_n1024",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m512_n1024",
            "value": 656288,
            "range": "± 5246",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u64/len64_edits4",
            "value": 284,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len64_edits4",
            "value": 483,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len64_edits4",
            "value": 2306,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k8/len64_edits4",
            "value": 1548,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len64_edits4",
            "value": 1548,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len64_edits4",
            "value": 5501,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len128_edits8",
            "value": 894,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len128_edits8",
            "value": 4418,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k16/len128_edits8",
            "value": 2813,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len128_edits8",
            "value": 2816,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len128_edits8",
            "value": 21208,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits8",
            "value": 17002,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k16/len512_edits8",
            "value": 10425,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits8",
            "value": 10422,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits8",
            "value": 329753,
            "range": "± 2634",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits32",
            "value": 16953,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k64/len512_edits32",
            "value": 17738,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits32",
            "value": 10436,
            "range": "± 308",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits32",
            "value": 329507,
            "range": "± 1151",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/dense",
            "value": 699,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/dense",
            "value": 3353,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/dense",
            "value": 5068,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/sparse",
            "value": 698,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/sparse",
            "value": 3352,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/sparse",
            "value": 2573,
            "range": "± 62",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "acannix@proton.me",
            "name": "Michal",
            "username": "mightsleep"
          },
          "committer": {
            "email": "acannix@proton.me",
            "name": "Michal",
            "username": "mightsleep"
          },
          "distinct": true,
          "id": "a37cacfefca30106ca605d966e2b8644c9d9ea7d",
          "message": "bench: making graphs actually usable",
          "timestamp": "2026-09-19T20:20:29+02:00",
          "tree_id": "80cf72732310a18be5314d773899cd6dbecf130f",
          "url": "https://github.com/mightsleep/hakmem/commit/a37cacfefca30106ca605d966e2b8644c9d9ea7d"
        },
        "date": 1789842421151,
        "tool": "cargo",
        "benches": [
          {
            "name": "select64_vs_broadword/hakmem",
            "value": 701,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/broadword::select1_raw",
            "value": 3096,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m16_n64",
            "value": 262,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m16_n64",
            "value": 425,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m16_n64",
            "value": 1342,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m16_n64",
            "value": 1815,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m16_n64",
            "value": 1441,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m16_n64",
            "value": 1038,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m16_n64",
            "value": 1417,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m32_n256",
            "value": 1005,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m32_n256",
            "value": 1439,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m32_n256",
            "value": 4407,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m32_n256",
            "value": 6347,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m32_n256",
            "value": 29466,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m32_n256",
            "value": 11534,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m32_n256",
            "value": 10493,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n256",
            "value": 1005,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n256",
            "value": 1474,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n256",
            "value": 5030,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n256",
            "value": 6678,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n256",
            "value": 32774,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n256",
            "value": 12824,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n256",
            "value": 20882,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n1024",
            "value": 3870,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n1024",
            "value": 5398,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n1024",
            "value": 15687,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n1024",
            "value": 24132,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n1024",
            "value": 403244,
            "range": "± 19665",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n1024",
            "value": 378431,
            "range": "± 1374",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n1024",
            "value": 82225,
            "range": "± 1125",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m128_n1024",
            "value": 5473,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m128_n1024",
            "value": 17184,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m128_n1024",
            "value": 24917,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m128_n1024",
            "value": 430696,
            "range": "± 19472",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m128_n1024",
            "value": 405650,
            "range": "± 18514",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m128_n1024",
            "value": 164288,
            "range": "± 582",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m256_n1024",
            "value": 19622,
            "range": "± 276",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m256_n1024",
            "value": 26128,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m256_n1024",
            "value": 479670,
            "range": "± 18858",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m256_n1024",
            "value": 442259,
            "range": "± 19324",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m256_n1024",
            "value": 328593,
            "range": "± 861",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m512_n1024",
            "value": 28493,
            "range": "± 376",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m512_n1024",
            "value": 572505,
            "range": "± 15996",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m512_n1024",
            "value": 532020,
            "range": "± 18574",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m512_n1024",
            "value": 656559,
            "range": "± 20562",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u64/len64_edits4",
            "value": 281,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len64_edits4",
            "value": 483,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len64_edits4",
            "value": 2295,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len64_edits4",
            "value": 1548,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len64_edits4",
            "value": 1548,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len64_edits4",
            "value": 5552,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len128_edits8",
            "value": 893,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len128_edits8",
            "value": 4442,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len128_edits8",
            "value": 2822,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len128_edits8",
            "value": 2816,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len128_edits8",
            "value": 21265,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits8",
            "value": 16981,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits8",
            "value": 10455,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits8",
            "value": 10464,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits8",
            "value": 329797,
            "range": "± 6087",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits32",
            "value": 16670,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits32",
            "value": 17723,
            "range": "± 619",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits32",
            "value": 10463,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits32",
            "value": 330310,
            "range": "± 1506",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/dense",
            "value": 699,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/dense",
            "value": 3353,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/dense",
            "value": 5069,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/sparse",
            "value": 696,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/sparse",
            "value": 3353,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/sparse",
            "value": 2559,
            "range": "± 35",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}