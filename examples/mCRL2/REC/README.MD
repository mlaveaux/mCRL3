These are various example rewrite systems from the Rewrite Engine Competition
[1]. Note that the REC specifications with META blocks have been omitted since
it is unclear how to expand them. The resulting mCRL2 specifications are used
for comparison studies between mCRL2 and the Sabre rewriter rewritten in Rust.
The benchmark results are shown below. Timeout of 10 minutes (600 seconds),
indicated by '-'.

The C++ code was compiled with GCC 11.4.0, and the Rust code with rustc 1.75.0
and benchmarks were performed on a i7-12800H, running Linux in WSL.

| Rewrite System | upstream jitty (4e954e) | upstream jittyc (4e954e) | innermost (40008e) |
| ---            | ---:                    | ---:                     | ---:               |
|           add8 |                   1 156 |                       14 |                411 |
|          add16 |                     294 |                       22 |                365 |
|          add32 |                     571 |                       10 |                892 |
|    benchexpr10 |                       9 |                        0 |                  6 |
|    benchexpr20 |                   8 421 |                      445 |              5 806 |
|    benchexpr22 |                  55 912 |                    1 768 |             26 291 |
|     benchsym10 |                      14 |                        0 |                  7 |
|     benchsym20 |                  10 799 |                      551 |              6 369 |
|     benchsym22 |                  30 960 |                    1 768 |             31 410 |
|    benchtree10 |                       - |                        - |                  - | 
|    benchtree20 |                       - |                        - |                  - |
|    benchtree22 |                       - |                        - |                  - |
|   binarysearch |                  58 935 |                    2 908 |            216 537 |
|   bubblesort10 |                       0 |                        0 |                  0 |
|   bubblesort20 |                       0 |                        0 |                  0 |
|  bubblesort100 |                      38 |                        1 |                 93 |
|  bubblesort720 |                  14 221 |                      374 |             35 175 |   
| bubblesort1000 |                  58 835 |                    1 057 |             97 913 |
|          calls |                       1 |                        0 |                  0 |
|         check1 |                       0 |                        0 |                  0 |
|         check2 |                       0 |                        0 |                  0 |
|     confluence |                       0 |                        0 |                  0 |
|        closure |                     431 |                       23 |                974 |
|           dart |                     100 |                        0 |                156 |
|          empty |                       0 |                        0 |                  0 |
|       evalexpr |                  12 015 |                      472 |             34 865 |
|        evalsym |                 560 323 |                   16 896 |                  - |
|       evaltree |                  28 798 |                        0 |             96 244 | 
|     factorial5 |                       0 |                        0 |                  0 |
|     factorial6 |                       1 |                        0 |                  0 |
|     factorial7 |                      11 |                        0 |                  1 |
|     factorial8 |                      90 |                        4 |                 12 |
|     factorial9 |                     736 |                       69 |                234 |
|          fib32 |                  28 743 |                    2 713 |            130 527 |
|       fibfree  |                       3 |                        0 |                  4 |
|    fibonacci05 |                       0 |                        0 |                  0 |
|    fibonacci18 |                      17 |                        0 |                  5 |
|    fibonacci19 |                      32 |                        0 |                  9 |
|    fibonacci20 |                      60 |                        1 |                 16 |
|    fibonacci21 |                      66 |                        1 |                 16 |
| garbagecollection |                    0 |                        0 |                  0 |
|         hanoi4 |                       0 |                        0 |                  0 |
|         hanoi8 |                       2 |                        0 |                  2 | 
|        hanoi12 |                      42 |                        1 |                 42 |
|        hanoi16 |                     813 |                       31 |                777 |
|        hanoi20 |                  14 807 |                      604 |             30 355 |
|         intnat |                      16 |                        1 |                 24 |
|          merge |                       0 |                        0 |                  3 |  
|    mergesort10 |                       0 |                        0 |                  0 |
|   mergesort100 |                       - |                        - |                  - |
|  mergesort1000 |                       - |                        - |                  - |
|  missionaries2 |                       2 |                        0 |                168 |
|  missionaries3 |                       9 |                        0 |                334 |
|            mma |                  36 246 |                    2 013 |            135 086 | 
|           mul8 |                   1 074 |                       32 |              1 183 |
|          mul16 |                   6 704 |                      205 |              8 321 |
|          mul32 |                  22 961 |                      752 |             29 368 |
|        natlist |                      33 |                        0 |                  - |
|       langton6 |                       - |                   35 152 |            307 055 |    
|       langton7 |                       - |                   52 209 |            488 722 |
|         logic3 |                       0 |                        0 |                  0 |
|        oddeven |                   1 372 |                    3 639 |              8 511 |
|  permutations6 |                      36 |                        2 |                 21 |
|  permutations7 |                       ? |                       97 |                965 |
|          omul8 |                  58 671 |                       38 |              1 205 |
|         omul32 |                  22 992 |                      672 |             27 634 |
|          order |                       0 |                        0 |                  0 |
|    quicksort10 |                      16 |                        0 |                 51 |
|   quicksort100 |                       - |                        - |                  - |
|  quicksort1000 |                       - |                        - |                  - |
| searchinconditions |                   0 |                        0 |                  0 |
| soundnessofparallelengines |           0 |                        0 |                  0 |
|      revnat100 |                       2 |                        0 |                  1 |
|     revnat1000 |                     240 |                       47 |                183 |
|    revnat10000 |                  25 160 |                    6 193 |             22 115 |
|        sieve20 |                       0 |                        0 |                  0 |
|       sieve100 |                      25 |                        0 |                172 |
|      sieve1000 |                   5 121 |                       79 |            145 827 |
|      sieve2000 |                  30 920 |                      444 |                  - |
|     sieve10000 |                       - |                    6 193 |                  - |
|          tak18 |                     150 |                        6 |                477 |
|          tak36 |                  12 578 |                      438 |             40 163 |
|  tautologyhard |                       0 |                        0 |                  0 |

 [1] Durán, F., Garavel, H. (2019). The Rewrite Engines Competitions: A RECtrospective. In: Beyer, D., Huisman, M., Kordon, F., Steffen, B. (eds) Tools and Algorithms for the Construction and Analysis of Systems. TACAS 2019. Lecture Notes in Computer Science(), vol 11429. Springer, Cham. https://doi.org/10.1007/978-3-030-17502-3_6