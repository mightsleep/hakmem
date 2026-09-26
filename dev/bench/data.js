window.BENCHMARK_DATA = {
  "lastUpdate": 1790439363894,
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
          "id": "8d7ed1fc8c69549873d6c61302bd046ddb6facdb",
          "message": "api: open Word with defaults, total runs/fills, rank rename, myers::distance, board8::slide, slice::positions, alloc Rank9Buf",
          "timestamp": "2026-09-19T22:32:44+02:00",
          "tree_id": "864c5cb57a0559ac4b0277d05e8163c70b51d110",
          "url": "https://github.com/mightsleep/hakmem/commit/8d7ed1fc8c69549873d6c61302bd046ddb6facdb"
        },
        "date": 1789850717279,
        "tool": "cargo",
        "benches": [
          {
            "name": "compact64/target/dense",
            "value": 1597,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/dense",
            "value": 9294,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/dense",
            "value": 40519,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/target/sparse",
            "value": 1597,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/sparse",
            "value": 9293,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/sparse",
            "value": 7770,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/dense",
            "value": 1597,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/dense",
            "value": 11496,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/dense",
            "value": 35356,
            "range": "± 1754",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/sparse",
            "value": 1597,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/sparse",
            "value": 11499,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/sparse",
            "value": 7373,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/hakmem",
            "value": 701,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/broadword::select1_raw",
            "value": 3093,
            "range": "± 4",
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
            "value": 426,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m16_n64",
            "value": 1341,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m16_n64",
            "value": 1818,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m16_n64",
            "value": 1448,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m16_n64",
            "value": 1040,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m16_n64",
            "value": 1329,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m32_n256",
            "value": 987,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m32_n256",
            "value": 1437,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m32_n256",
            "value": 4406,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m32_n256",
            "value": 6370,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m32_n256",
            "value": 31387,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m32_n256",
            "value": 11555,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m32_n256",
            "value": 9766,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n256",
            "value": 1004,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n256",
            "value": 1474,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n256",
            "value": 5033,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n256",
            "value": 6699,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n256",
            "value": 34660,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n256",
            "value": 12852,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n256",
            "value": 19449,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n1024",
            "value": 3871,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n1024",
            "value": 5398,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n1024",
            "value": 15697,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n1024",
            "value": 24393,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n1024",
            "value": 417041,
            "range": "± 15092",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n1024",
            "value": 420051,
            "range": "± 14049",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n1024",
            "value": 76300,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m128_n1024",
            "value": 5476,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m128_n1024",
            "value": 17193,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m128_n1024",
            "value": 25105,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m128_n1024",
            "value": 440916,
            "range": "± 16748",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m128_n1024",
            "value": 429062,
            "range": "± 28091",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m128_n1024",
            "value": 152603,
            "range": "± 421",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m256_n1024",
            "value": 19631,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m256_n1024",
            "value": 26180,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m256_n1024",
            "value": 496874,
            "range": "± 8983",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m256_n1024",
            "value": 465799,
            "range": "± 15476",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m256_n1024",
            "value": 306139,
            "range": "± 2338",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m512_n1024",
            "value": 29095,
            "range": "± 987",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m512_n1024",
            "value": 631238,
            "range": "± 7834",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m512_n1024",
            "value": 597446,
            "range": "± 17997",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m512_n1024",
            "value": 610856,
            "range": "± 4544",
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
            "value": 484,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len64_edits4",
            "value": 2290,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len64_edits4",
            "value": 1545,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len64_edits4",
            "value": 1546,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len64_edits4",
            "value": 5211,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len128_edits8",
            "value": 888,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len128_edits8",
            "value": 4419,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len128_edits8",
            "value": 2809,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len128_edits8",
            "value": 2811,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len128_edits8",
            "value": 19863,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits8",
            "value": 16928,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits8",
            "value": 10399,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits8",
            "value": 10397,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits8",
            "value": 307468,
            "range": "± 1217",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits32",
            "value": 16748,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits32",
            "value": 17714,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits32",
            "value": 10400,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits32",
            "value": 309797,
            "range": "± 1382",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/dense",
            "value": 2103,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/dense",
            "value": 1725,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/dense",
            "value": 3577,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/dense",
            "value": 5223,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/dense",
            "value": 8694,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/dense",
            "value": 11029,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/dense",
            "value": 14398,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/dense",
            "value": 13421,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/sparse",
            "value": 2104,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/sparse",
            "value": 1726,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/sparse",
            "value": 3576,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/sparse",
            "value": 6184,
            "range": "± 491",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/sparse",
            "value": 3992,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/sparse",
            "value": 4946,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/sparse",
            "value": 21155,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/sparse",
            "value": 20004,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/dense",
            "value": 699,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/dense",
            "value": 3354,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/dense",
            "value": 5077,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/sparse",
            "value": 694,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/sparse",
            "value": 3354,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/sparse",
            "value": 2587,
            "range": "± 14",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Michal",
            "username": "mightsleep",
            "email": "acannix@proton.me"
          },
          "committer": {
            "name": "Michal",
            "username": "mightsleep",
            "email": "acannix@proton.me"
          },
          "id": "ae4e9959d9f39767a2b07637ebfa4d953909da57",
          "message": "0.1.0: changelog dated, linked from README",
          "timestamp": "2026-09-19T20:52:15Z",
          "url": "https://github.com/mightsleep/hakmem/commit/ae4e9959d9f39767a2b07637ebfa4d953909da57"
        },
        "date": 1789965442350,
        "tool": "cargo",
        "benches": [
          {
            "name": "compact64/target/dense",
            "value": 1093,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/dense",
            "value": 6682,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/dense",
            "value": 24423,
            "range": "± 648",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/target/sparse",
            "value": 1048,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/sparse",
            "value": 6511,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/sparse",
            "value": 3970,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/dense",
            "value": 1021,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/dense",
            "value": 7606,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/dense",
            "value": 22787,
            "range": "± 742",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/sparse",
            "value": 1023,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/sparse",
            "value": 7542,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/sparse",
            "value": 4046,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/hakmem",
            "value": 785,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/broadword::select1_raw",
            "value": 3388,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m16_n64",
            "value": 184,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m16_n64",
            "value": 281,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m16_n64",
            "value": 880,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m16_n64",
            "value": 1186,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m16_n64",
            "value": 1055,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m16_n64",
            "value": 857,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m16_n64",
            "value": 1034,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m32_n256",
            "value": 747,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m32_n256",
            "value": 1037,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m32_n256",
            "value": 3113,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m32_n256",
            "value": 4699,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m32_n256",
            "value": 24840,
            "range": "± 568",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m32_n256",
            "value": 8225,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m32_n256",
            "value": 7610,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n256",
            "value": 786,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n256",
            "value": 1143,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n256",
            "value": 3130,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n256",
            "value": 4711,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n256",
            "value": 26166,
            "range": "± 843",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n256",
            "value": 8701,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n256",
            "value": 15245,
            "range": "± 516",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n1024",
            "value": 2927,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n1024",
            "value": 3902,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n1024",
            "value": 11027,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n1024",
            "value": 17087,
            "range": "± 1061",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n1024",
            "value": 309032,
            "range": "± 8126",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n1024",
            "value": 303593,
            "range": "± 10702",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n1024",
            "value": 55143,
            "range": "± 796",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m128_n1024",
            "value": 4044,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m128_n1024",
            "value": 11485,
            "range": "± 394",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m128_n1024",
            "value": 17330,
            "range": "± 834",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m128_n1024",
            "value": 333660,
            "range": "± 17870",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m128_n1024",
            "value": 322809,
            "range": "± 3854",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m128_n1024",
            "value": 111171,
            "range": "± 2365",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m256_n1024",
            "value": 12434,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m256_n1024",
            "value": 18692,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m256_n1024",
            "value": 366584,
            "range": "± 7871",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m256_n1024",
            "value": 365350,
            "range": "± 5239",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m256_n1024",
            "value": 224209,
            "range": "± 5933",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m512_n1024",
            "value": 20179,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m512_n1024",
            "value": 433633,
            "range": "± 14842",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m512_n1024",
            "value": 434740,
            "range": "± 17386",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m512_n1024",
            "value": 434024,
            "range": "± 8079",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u64/len64_edits4",
            "value": 206,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len64_edits4",
            "value": 315,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len64_edits4",
            "value": 1556,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len64_edits4",
            "value": 908,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len64_edits4",
            "value": 901,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len64_edits4",
            "value": 3982,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len128_edits8",
            "value": 620,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len128_edits8",
            "value": 3146,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len128_edits8",
            "value": 1720,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len128_edits8",
            "value": 1604,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len128_edits8",
            "value": 14824,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits8",
            "value": 12234,
            "range": "± 497",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits8",
            "value": 5821,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits8",
            "value": 5787,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits8",
            "value": 219676,
            "range": "± 12866",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits32",
            "value": 12031,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits32",
            "value": 13391,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits32",
            "value": 5795,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits32",
            "value": 226315,
            "range": "± 10096",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/dense",
            "value": 1630,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/dense",
            "value": 857,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/dense",
            "value": 2568,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/dense",
            "value": 3336,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/dense",
            "value": 6464,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/dense",
            "value": 6673,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/dense",
            "value": 10088,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/dense",
            "value": 9500,
            "range": "± 454",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/sparse",
            "value": 1443,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/sparse",
            "value": 907,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/sparse",
            "value": 2575,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/sparse",
            "value": 3310,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/sparse",
            "value": 2756,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/sparse",
            "value": 3593,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/sparse",
            "value": 14814,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/sparse",
            "value": 13264,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/dense",
            "value": 838,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/dense",
            "value": 2714,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/dense",
            "value": 2309,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/sparse",
            "value": 765,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/sparse",
            "value": 2649,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/sparse",
            "value": 2287,
            "range": "± 81",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "acannix@proton.me",
            "name": "mightsleep",
            "username": "mightsleep"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "cdca6002c2f18edbab4848fd2977d02325371136",
          "message": "Merge pull request #6 from mightsleep/hilbert\n\nmaybe 0.2.0: A Nocturne of Curves; lanes, affine, Hilbert 2D and 3D",
          "timestamp": "2026-09-25T00:36:31+02:00",
          "tree_id": "610e1b3ee64ad072b2ba92956b6132b645053955",
          "url": "https://github.com/mightsleep/hakmem/commit/cdca6002c2f18edbab4848fd2977d02325371136"
        },
        "date": 1790289909097,
        "tool": "cargo",
        "benches": [
          {
            "name": "compact64/target/dense",
            "value": 1805,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/dense",
            "value": 9749,
            "range": "± 417",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/dense",
            "value": 24704,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/target/sparse",
            "value": 1804,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/sparse",
            "value": 9741,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/sparse",
            "value": 6431,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/dense",
            "value": 1805,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/dense",
            "value": 12964,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/dense",
            "value": 25725,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/sparse",
            "value": 1804,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/sparse",
            "value": 12976,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/sparse",
            "value": 8284,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/hakmem",
            "value": 710,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/broadword::select1_raw",
            "value": 3354,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m16_n64",
            "value": 290,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m16_n64",
            "value": 432,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m16_n64",
            "value": 1525,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m16_n64",
            "value": 1908,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m16_n64",
            "value": 1550,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m16_n64",
            "value": 1151,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m16_n64",
            "value": 1368,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m32_n256",
            "value": 1113,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m32_n256",
            "value": 1523,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m32_n256",
            "value": 5109,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m32_n256",
            "value": 6655,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m32_n256",
            "value": 32100,
            "range": "± 596",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m32_n256",
            "value": 12513,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m32_n256",
            "value": 10852,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n256",
            "value": 1140,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n256",
            "value": 1568,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n256",
            "value": 5784,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n256",
            "value": 7047,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n256",
            "value": 35926,
            "range": "± 1484",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n256",
            "value": 13932,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n256",
            "value": 21622,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n1024",
            "value": 4372,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n1024",
            "value": 5890,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n1024",
            "value": 18369,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n1024",
            "value": 25260,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n1024",
            "value": 425837,
            "range": "± 14246",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n1024",
            "value": 415715,
            "range": "± 6187",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n1024",
            "value": 86050,
            "range": "± 3841",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m128_n1024",
            "value": 6097,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m128_n1024",
            "value": 19987,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m128_n1024",
            "value": 26023,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m128_n1024",
            "value": 457945,
            "range": "± 16604",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m128_n1024",
            "value": 454530,
            "range": "± 7087",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m128_n1024",
            "value": 173516,
            "range": "± 7547",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m256_n1024",
            "value": 22943,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m256_n1024",
            "value": 27518,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m256_n1024",
            "value": 496972,
            "range": "± 3870",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m256_n1024",
            "value": 463677,
            "range": "± 19150",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m256_n1024",
            "value": 344274,
            "range": "± 8200",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m512_n1024",
            "value": 30454,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m512_n1024",
            "value": 595062,
            "range": "± 15651",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m512_n1024",
            "value": 581532,
            "range": "± 15490",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m512_n1024",
            "value": 691947,
            "range": "± 16390",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u64/len64_edits4",
            "value": 315,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len64_edits4",
            "value": 501,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len64_edits4",
            "value": 2486,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len64_edits4",
            "value": 1623,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len64_edits4",
            "value": 1624,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len64_edits4",
            "value": 5386,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len128_edits8",
            "value": 948,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len128_edits8",
            "value": 4766,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len128_edits8",
            "value": 2975,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len128_edits8",
            "value": 2975,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len128_edits8",
            "value": 21306,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits8",
            "value": 18202,
            "range": "± 266",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits8",
            "value": 11084,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits8",
            "value": 11086,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits8",
            "value": 345224,
            "range": "± 834",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits32",
            "value": 18297,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits32",
            "value": 19961,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits32",
            "value": 11085,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits32",
            "value": 344267,
            "range": "± 5442",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/dense",
            "value": 2460,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/dense",
            "value": 1933,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/dense",
            "value": 3675,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/dense",
            "value": 5684,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/dense",
            "value": 10181,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/dense",
            "value": 12876,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/dense",
            "value": 16979,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/dense",
            "value": 14974,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/sparse",
            "value": 2463,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/sparse",
            "value": 1940,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/sparse",
            "value": 3671,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/sparse",
            "value": 5682,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/sparse",
            "value": 3796,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/sparse",
            "value": 4884,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/sparse",
            "value": 22074,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/sparse",
            "value": 21169,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/dense",
            "value": 711,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/dense",
            "value": 6833,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/dense",
            "value": 3735,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/sparse",
            "value": 711,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/sparse",
            "value": 6832,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/sparse",
            "value": 2730,
            "range": "± 59",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "acannix@proton.me",
            "name": "mightsleep",
            "username": "mightsleep"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a1fbbc80b97f3f6bd2187a437240e382cbfc1f64",
          "message": "Merge pull request #8 from mightsleep/isa\n\nisa: instruction sets as values, and checks that read what a user's crate gets",
          "timestamp": "2026-09-26T02:15:29+02:00",
          "tree_id": "fd9b95b9183529e88f5fa6730a3e24308746f9ed",
          "url": "https://github.com/mightsleep/hakmem/commit/a1fbbc80b97f3f6bd2187a437240e382cbfc1f64"
        },
        "date": 1790382241235,
        "tool": "cargo",
        "benches": [
          {
            "name": "compact64/target/dense",
            "value": 2871,
            "range": "± 433",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/dense",
            "value": 9216,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/dense",
            "value": 40771,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/target/sparse",
            "value": 2872,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/sparse",
            "value": 9218,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/sparse",
            "value": 7678,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/dense",
            "value": 2859,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/dense",
            "value": 11691,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/dense",
            "value": 35588,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/sparse",
            "value": 1924,
            "range": "± 478",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/sparse",
            "value": 11683,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/sparse",
            "value": 8649,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/hakmem",
            "value": 698,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/broadword::select1_raw",
            "value": 3094,
            "range": "± 10",
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
            "value": 426,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m16_n64",
            "value": 1343,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m16_n64",
            "value": 1824,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m16_n64",
            "value": 1455,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m16_n64",
            "value": 1041,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m16_n64",
            "value": 1331,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m32_n256",
            "value": 1004,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m32_n256",
            "value": 1437,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m32_n256",
            "value": 4406,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m32_n256",
            "value": 6379,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m32_n256",
            "value": 30923,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m32_n256",
            "value": 11571,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m32_n256",
            "value": 9782,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n256",
            "value": 1006,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n256",
            "value": 1475,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n256",
            "value": 5036,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n256",
            "value": 6718,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n256",
            "value": 37549,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n256",
            "value": 12862,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n256",
            "value": 19456,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n1024",
            "value": 3873,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n1024",
            "value": 5399,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n1024",
            "value": 15687,
            "range": "± 497",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n1024",
            "value": 24148,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n1024",
            "value": 418632,
            "range": "± 22854",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n1024",
            "value": 400340,
            "range": "± 11486",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n1024",
            "value": 76336,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m128_n1024",
            "value": 5491,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m128_n1024",
            "value": 17199,
            "range": "± 1193",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m128_n1024",
            "value": 25072,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m128_n1024",
            "value": 443629,
            "range": "± 18476",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m128_n1024",
            "value": 447123,
            "range": "± 18834",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m128_n1024",
            "value": 152813,
            "range": "± 845",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m256_n1024",
            "value": 19625,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m256_n1024",
            "value": 26254,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m256_n1024",
            "value": 493847,
            "range": "± 16693",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m256_n1024",
            "value": 470773,
            "range": "± 32924",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m256_n1024",
            "value": 307100,
            "range": "± 9912",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m512_n1024",
            "value": 28726,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m512_n1024",
            "value": 630566,
            "range": "± 14481",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m512_n1024",
            "value": 565968,
            "range": "± 20816",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m512_n1024",
            "value": 611678,
            "range": "± 23661",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u64/len64_edits4",
            "value": 281,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len64_edits4",
            "value": 484,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len64_edits4",
            "value": 2309,
            "range": "± 40",
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
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len64_edits4",
            "value": 5211,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len128_edits8",
            "value": 892,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len128_edits8",
            "value": 4486,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len128_edits8",
            "value": 2813,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len128_edits8",
            "value": 2814,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len128_edits8",
            "value": 19863,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits8",
            "value": 16701,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits8",
            "value": 10412,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits8",
            "value": 10426,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits8",
            "value": 307734,
            "range": "± 1439",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits32",
            "value": 16863,
            "range": "± 393",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits32",
            "value": 17733,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits32",
            "value": 10418,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits32",
            "value": 307230,
            "range": "± 1492",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/dense",
            "value": 2382,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/dense",
            "value": 1731,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/dense",
            "value": 3580,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/dense",
            "value": 5232,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/dense",
            "value": 8826,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/dense",
            "value": 11241,
            "range": "± 522",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/dense",
            "value": 14461,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/dense",
            "value": 13425,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/sparse",
            "value": 2383,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/sparse",
            "value": 1730,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/sparse",
            "value": 3570,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/sparse",
            "value": 5236,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/sparse",
            "value": 3868,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/sparse",
            "value": 4208,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/sparse",
            "value": 21150,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/sparse",
            "value": 20044,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/dense",
            "value": 696,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/dense",
            "value": 5782,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/dense",
            "value": 5072,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/sparse",
            "value": 700,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/sparse",
            "value": 5772,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/sparse",
            "value": 2567,
            "range": "± 16",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "acannix@proton.me",
            "name": "mightsleep",
            "username": "mightsleep"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "50719ec9a33cabcd70357117c83c9d5535631898",
          "message": "Merge pull request #9 from mightsleep/doc",
          "timestamp": "2026-09-26T03:57:35+02:00",
          "tree_id": "ee4ee1ae697a09875ca60dfdbc6fe024f31d0c59",
          "url": "https://github.com/mightsleep/hakmem/commit/50719ec9a33cabcd70357117c83c9d5535631898"
        },
        "date": 1790388347930,
        "tool": "cargo",
        "benches": [
          {
            "name": "compact64/target/dense",
            "value": 1916,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/dense",
            "value": 9204,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/dense",
            "value": 40720,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/target/sparse",
            "value": 2873,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/sparse",
            "value": 9200,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/sparse",
            "value": 7665,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/dense",
            "value": 2869,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/dense",
            "value": 11681,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/dense",
            "value": 35628,
            "range": "± 2558",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/sparse",
            "value": 1915,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/sparse",
            "value": 11671,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/sparse",
            "value": 7685,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/hakmem",
            "value": 708,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/broadword::select1_raw",
            "value": 3103,
            "range": "± 8",
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
            "value": 428,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m16_n64",
            "value": 1342,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m16_n64",
            "value": 1823,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m16_n64",
            "value": 1451,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m16_n64",
            "value": 1039,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m16_n64",
            "value": 1329,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m32_n256",
            "value": 1004,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m32_n256",
            "value": 1434,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m32_n256",
            "value": 4404,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m32_n256",
            "value": 6372,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m32_n256",
            "value": 31453,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m32_n256",
            "value": 11533,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m32_n256",
            "value": 9777,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n256",
            "value": 1004,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n256",
            "value": 1474,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n256",
            "value": 5026,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n256",
            "value": 6711,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n256",
            "value": 34236,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n256",
            "value": 12823,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n256",
            "value": 19444,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n1024",
            "value": 3871,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n1024",
            "value": 5413,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n1024",
            "value": 15676,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n1024",
            "value": 24153,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n1024",
            "value": 423135,
            "range": "± 16081",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n1024",
            "value": 399457,
            "range": "± 16582",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n1024",
            "value": 76257,
            "range": "± 563",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m128_n1024",
            "value": 5489,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m128_n1024",
            "value": 17168,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m128_n1024",
            "value": 25034,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m128_n1024",
            "value": 442529,
            "range": "± 16508",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m128_n1024",
            "value": 435177,
            "range": "± 17336",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m128_n1024",
            "value": 152651,
            "range": "± 483",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m256_n1024",
            "value": 19579,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m256_n1024",
            "value": 26248,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m256_n1024",
            "value": 490905,
            "range": "± 8141",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m256_n1024",
            "value": 470138,
            "range": "± 18282",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m256_n1024",
            "value": 306159,
            "range": "± 770",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m512_n1024",
            "value": 28672,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m512_n1024",
            "value": 589829,
            "range": "± 15680",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m512_n1024",
            "value": 568857,
            "range": "± 15980",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m512_n1024",
            "value": 610494,
            "range": "± 17437",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u64/len64_edits4",
            "value": 281,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len64_edits4",
            "value": 486,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len64_edits4",
            "value": 2311,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len64_edits4",
            "value": 1551,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len64_edits4",
            "value": 1551,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len64_edits4",
            "value": 5210,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len128_edits8",
            "value": 893,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len128_edits8",
            "value": 4430,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len128_edits8",
            "value": 2812,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len128_edits8",
            "value": 2812,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len128_edits8",
            "value": 19858,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits8",
            "value": 16697,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits8",
            "value": 10403,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits8",
            "value": 10404,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits8",
            "value": 307697,
            "range": "± 1483",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits32",
            "value": 16712,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits32",
            "value": 17726,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits32",
            "value": 10410,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits32",
            "value": 306983,
            "range": "± 1259",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/dense",
            "value": 2377,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/dense",
            "value": 1724,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/dense",
            "value": 3564,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/dense",
            "value": 5224,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/dense",
            "value": 8813,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/dense",
            "value": 11193,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/dense",
            "value": 14471,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/dense",
            "value": 13404,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/sparse",
            "value": 2379,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/sparse",
            "value": 1728,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/sparse",
            "value": 3563,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/sparse",
            "value": 5224,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/sparse",
            "value": 3863,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/sparse",
            "value": 4201,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/sparse",
            "value": 21131,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/sparse",
            "value": 19992,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/dense",
            "value": 700,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/dense",
            "value": 5768,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/dense",
            "value": 5067,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/sparse",
            "value": 698,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/sparse",
            "value": 5766,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/sparse",
            "value": 2574,
            "range": "± 31",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "acannix@proton.me",
            "name": "mightsleep",
            "username": "mightsleep"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f9b0c34a9f5792bcdbf2d9139cd4a8e8c1eb79a3",
          "message": "Merge pull request #10 from mightsleep/nix-ci\n\ngetting better ci in nix",
          "timestamp": "2026-09-26T18:07:58+02:00",
          "tree_id": "45461c8c41cfa34f6480012b3476a370f4e95ea5",
          "url": "https://github.com/mightsleep/hakmem/commit/f9b0c34a9f5792bcdbf2d9139cd4a8e8c1eb79a3"
        },
        "date": 1790439362331,
        "tool": "cargo",
        "benches": [
          {
            "name": "compact64/target/dense",
            "value": 1673,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/dense",
            "value": 8740,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/dense",
            "value": 33674,
            "range": "± 430",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/target/sparse",
            "value": 1680,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/broadword/sparse",
            "value": 8776,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "compact64/loop/sparse",
            "value": 8125,
            "range": "± 472",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/dense",
            "value": 1674,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/dense",
            "value": 9925,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/dense",
            "value": 29065,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/target/sparse",
            "value": 1672,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/broadword/sparse",
            "value": 9938,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "expand64/loop/sparse",
            "value": 5233,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/hakmem",
            "value": 1005,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "select64_vs_broadword/broadword::select1_raw",
            "value": 4412,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m16_n64",
            "value": 244,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m16_n64",
            "value": 359,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m16_n64",
            "value": 1063,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m16_n64",
            "value": 1600,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m16_n64",
            "value": 1358,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m16_n64",
            "value": 1080,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m16_n64",
            "value": 1103,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m32_n256",
            "value": 970,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m32_n256",
            "value": 1338,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m32_n256",
            "value": 3860,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m32_n256",
            "value": 5832,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m32_n256",
            "value": 29418,
            "range": "± 429",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m32_n256",
            "value": 10404,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m32_n256",
            "value": 8856,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n256",
            "value": 993,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n256",
            "value": 1366,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n256",
            "value": 4129,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n256",
            "value": 6171,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n256",
            "value": 32544,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n256",
            "value": 11550,
            "range": "± 390",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n256",
            "value": 17598,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u64/m64_n1024",
            "value": 3845,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m64_n1024",
            "value": 5174,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m64_n1024",
            "value": 14841,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m64_n1024",
            "value": 22402,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m64_n1024",
            "value": 392875,
            "range": "± 8553",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m64_n1024",
            "value": 378785,
            "range": "± 4345",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m64_n1024",
            "value": 68585,
            "range": "± 693",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_u128/m128_n1024",
            "value": 5233,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m128_n1024",
            "value": 15392,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m128_n1024",
            "value": 22982,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m128_n1024",
            "value": 414167,
            "range": "± 5206",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m128_n1024",
            "value": 401920,
            "range": "± 9011",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m128_n1024",
            "value": 136575,
            "range": "± 782",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide4/m256_n1024",
            "value": 16301,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m256_n1024",
            "value": 24469,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m256_n1024",
            "value": 460905,
            "range": "± 6301",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m256_n1024",
            "value": 444356,
            "range": "± 13301",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m256_n1024",
            "value": 274055,
            "range": "± 1447",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/hakmem_wide8/m512_n1024",
            "value": 26921,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel/m512_n1024",
            "value": 557083,
            "range": "± 18289",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/triple_accel_exp/m512_n1024",
            "value": 532104,
            "range": "± 10586",
            "unit": "ns/iter"
          },
          {
            "name": "edit_distance/strsim/m512_n1024",
            "value": 547963,
            "range": "± 15686",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u64/len64_edits4",
            "value": 264,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len64_edits4",
            "value": 408,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len64_edits4",
            "value": 2065,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len64_edits4",
            "value": 1189,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len64_edits4",
            "value": 1190,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len64_edits4",
            "value": 4305,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_u128/len128_edits8",
            "value": 791,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len128_edits8",
            "value": 4109,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len128_edits8",
            "value": 2121,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len128_edits8",
            "value": 2127,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len128_edits8",
            "value": 18170,
            "range": "± 481",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits8",
            "value": 15962,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits8",
            "value": 7721,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits8",
            "value": 7723,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits8",
            "value": 275401,
            "range": "± 1148",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/hakmem_wide8/len512_edits32",
            "value": 15954,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_simd_k/len512_edits32",
            "value": 17416,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/triple_accel_exp/len512_edits32",
            "value": 7730,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "similar_strings/strsim/len512_edits32",
            "value": 275237,
            "range": "± 900",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/dense",
            "value": 1865,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/dense",
            "value": 1147,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/dense",
            "value": 3460,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/dense",
            "value": 4405,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/dense",
            "value": 7916,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/dense",
            "value": 8826,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/dense",
            "value": 13591,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/dense",
            "value": 12402,
            "range": "± 424",
            "unit": "ns/iter"
          },
          {
            "name": "rank/hakmem/sparse",
            "value": 1861,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sux/sparse",
            "value": 1127,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "rank/sucds/sparse",
            "value": 3458,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "rank/vers/sparse",
            "value": 4397,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "select/hakmem/sparse",
            "value": 2847,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "select/sux/sparse",
            "value": 3109,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "select/sucds/sparse",
            "value": 19226,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "select/vers/sparse",
            "value": 17478,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/dense",
            "value": 1003,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/dense",
            "value": 3786,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/dense",
            "value": 3100,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "select64/target/sparse",
            "value": 1001,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "select64/broadword/sparse",
            "value": 3788,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "select64/loop/sparse",
            "value": 3164,
            "range": "± 41",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}