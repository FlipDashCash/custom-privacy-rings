Measured with the frame encoders, SPP's account builders and the v1 compiler. Ceilings 4096 B, 64 accounts, 12 signatures.
min: tracking root, no co-signer, no leg, no fee, the payer an owner where it may be. full: own, curator and tracking roots, one co-signer, 4 SOL legs, a fee, an outside payer, a body on every slot and one exit. data: a data hash in every record. ring: a ring data hash in every non-exit record. program: the test program forwards the transfer under its proof and takes no leg, its PDA the one owner, and the co-signer approves at the top level. The owner run is the ceilings' bound under full, the key's rows bound it apart. A shape's every input and output slot is published.
Every row is one transaction. fits: the row clears the transaction and CPI ceilings. limit: what refuses a row, rail for a shape the rail has no SPP key for, else the excess in accounts, signatures, bytes or CPI data at that row's load. A missing catalog key has no encoded measurement.
| shape | rail | load | catalog key | bytes | accounts | signatures | owner run | frame | SPP CPI | fits | limit |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1x2 | confidential signer | min | yes | 1259 | 9 | 1 | - | 843 | 857 | yes |  |
| 1x2 | confidential signer | full | yes | 1918 | 19 | 3 | 1 | 1034 | 1006 | yes |  |
| 1x2 | confidential signer | full+spl | yes | 2286 | 26 | 3 | - | 1166 | 1010 | yes |  |
| 1x2 | confidential signer | full+data | yes | 2023 | 19 | 3 | - | 1139 | 1107 | yes |  |
| 1x2 | confidential signer | full+reply | yes | 1918 | 19 | 3 | - | 1034 | 1006 | yes |  |
| 1x2 | confidential signer | full+2tree | yes | 1956 | 20 | 3 | - | 1039 | 1010 | yes |  |
| 1x2 | confidential signer | full+data+ring | yes | 2055 | 19 | 3 | - | 1171 | 1139 | yes |  |
| 1x2 | confidential signer | full+data+program | yes | 1990 | 16 | 2 | - | 1103 | 1071 | yes |  |
| 1x2 | confidential signer | full+data+ring+program | yes | 2022 | 16 | 2 | - | 1135 | 1103 | yes |  |
| 1x2 | confidential in-proof | min | yes | 1354 | 9 | 1 | - | 939 | 954 | yes |  |
| 1x2 | confidential in-proof | full | yes | 1917 | 18 | 2 | 1 | 1130 | 1103 | yes |  |
| 1x2 | confidential in-proof | full+spl | yes | 2285 | 25 | 2 | - | 1262 | 1107 | yes |  |
| 1x2 | confidential in-proof | full+data | yes | 2022 | 18 | 2 | - | 1235 | 1204 | yes |  |
| 1x2 | confidential in-proof | full+reply | yes | 1917 | 18 | 2 | - | 1130 | 1103 | yes |  |
| 1x2 | confidential in-proof | full+2tree | yes | 1955 | 19 | 2 | - | 1135 | 1107 | yes |  |
| 1x2 | confidential in-proof | full+data+ring | yes | 2054 | 18 | 2 | - | 1267 | 1236 | yes |  |
| 1x2 | anonymous hidden | min | yes | 1386 | 9 | 1 | - | 971 | 1026 | yes |  |
| 1x2 | anonymous hidden | full | yes | 1853 | 15 | 2 | 1 | 1162 | 1175 | yes |  |
| 1x2 | anonymous hidden | full+spl | yes | 2221 | 22 | 2 | - | 1294 | 1179 | yes |  |
| 1x2 | anonymous hidden | full+data | yes | 1958 | 15 | 2 | - | 1267 | 1276 | yes |  |
| 1x2 | anonymous hidden | full+reply | yes | 1952 | 15 | 2 | - | 1261 | 1272 | yes |  |
| 1x2 | anonymous hidden | full+2tree | yes | 1891 | 16 | 2 | - | 1167 | 1179 | yes |  |
| 1x2 | anonymous hidden | full+data+ring | yes | 1990 | 15 | 2 | - | 1299 | 1308 | yes |  |
| 1x2 | anonymous public owners | min | yes | 1323 | 9 | 1 | - | 907 | 929 | yes |  |
| 1x2 | anonymous public owners | full | yes | 1982 | 19 | 3 | 1 | 1098 | 1078 | yes |  |
| 1x2 | anonymous public owners | full+spl | yes | 2350 | 26 | 3 | - | 1230 | 1082 | yes |  |
| 1x2 | anonymous public owners | full+data | yes | 2087 | 19 | 3 | - | 1203 | 1179 | yes |  |
| 1x2 | anonymous public owners | full+reply | yes | 2081 | 19 | 3 | - | 1197 | 1175 | yes |  |
| 1x2 | anonymous public owners | full+2tree | yes | 2020 | 20 | 3 | - | 1103 | 1082 | yes |  |
| 1x2 | anonymous public owners | full+data+ring | yes | 2119 | 19 | 3 | - | 1235 | 1211 | yes |  |
| 1x2 | anonymous public owners | full+data+program | yes | 2054 | 16 | 2 | - | 1167 | 1143 | yes |  |
| 1x2 | anonymous public owners | full+data+ring+program | yes | 2086 | 16 | 2 | - | 1199 | 1175 | yes |  |
| 1x2 | confidential entry | min | yes | 1403 | 9 | 1 | - | 987 | 1001 | yes |  |
| 1x2 | confidential entry | full | yes | 2062 | 19 | 3 | 1 | 1178 | 1150 | yes |  |
| 1x2 | confidential entry | full+spl | yes | 2430 | 26 | 3 | - | 1310 | 1154 | yes |  |
| 1x2 | confidential entry | full+data | yes | 2167 | 19 | 3 | - | 1283 | 1251 | yes |  |
| 1x2 | confidential entry | full+reply | yes | 2062 | 19 | 3 | - | 1178 | 1150 | yes |  |
| 1x2 | confidential entry | full+2tree | yes | 2100 | 20 | 3 | - | 1183 | 1154 | yes |  |
| 1x2 | confidential entry | full+data+ring | yes | 2199 | 19 | 3 | - | 1315 | 1283 | yes |  |
| 1x2 | anonymous entry | min | yes | 1562 | 9 | 1 | - | 1147 | 1170 | yes |  |
| 1x2 | anonymous entry | full | yes | 2029 | 15 | 2 | 1 | 1338 | 1319 | yes |  |
| 1x2 | anonymous entry | full+spl | yes | 2397 | 22 | 2 | - | 1470 | 1323 | yes |  |
| 1x2 | anonymous entry | full+data | yes | 2134 | 15 | 2 | - | 1443 | 1420 | yes |  |
| 1x2 | anonymous entry | full+reply | yes | 2128 | 15 | 2 | - | 1437 | 1416 | yes |  |
| 1x2 | anonymous entry | full+2tree | yes | 2067 | 16 | 2 | - | 1343 | 1323 | yes |  |
| 1x2 | anonymous entry | full+data+ring | yes | 2166 | 15 | 2 | - | 1475 | 1452 | yes |  |
| 1x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 1x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 1x4 | confidential signer | min | yes | 1613 | 9 | 1 | - | 1197 | 1283 | yes |  |
| 1x4 | confidential signer | full | yes | 2272 | 19 | 3 | 1 | 1388 | 1432 | yes |  |
| 1x4 | confidential signer | full+spl | yes | 2640 | 26 | 3 | - | 1520 | 1436 | yes |  |
| 1x4 | confidential signer | full+data | yes | 2449 | 19 | 3 | - | 1565 | 1601 | yes |  |
| 1x4 | confidential signer | full+reply | yes | 2272 | 19 | 3 | - | 1388 | 1432 | yes |  |
| 1x4 | confidential signer | full+2tree | yes | 2310 | 20 | 3 | - | 1393 | 1436 | yes |  |
| 1x4 | confidential signer | full+data+ring | yes | 2545 | 19 | 3 | - | 1661 | 1697 | yes |  |
| 1x4 | confidential signer | full+data+program | yes | 2416 | 16 | 2 | - | 1529 | 1565 | yes |  |
| 1x4 | confidential signer | full+data+ring+program | yes | 2512 | 16 | 2 | - | 1625 | 1661 | yes |  |
| 1x4 | confidential in-proof | min | yes | 1708 | 9 | 1 | - | 1293 | 1380 | yes |  |
| 1x4 | confidential in-proof | full | yes | 2271 | 18 | 2 | 1 | 1484 | 1529 | yes |  |
| 1x4 | confidential in-proof | full+spl | yes | 2639 | 25 | 2 | - | 1616 | 1533 | yes |  |
| 1x4 | confidential in-proof | full+data | yes | 2448 | 18 | 2 | - | 1661 | 1698 | yes |  |
| 1x4 | confidential in-proof | full+reply | yes | 2271 | 18 | 2 | - | 1484 | 1529 | yes |  |
| 1x4 | confidential in-proof | full+2tree | yes | 2309 | 19 | 2 | - | 1489 | 1533 | yes |  |
| 1x4 | confidential in-proof | full+data+ring | yes | 2544 | 18 | 2 | - | 1757 | 1794 | yes |  |
| 1x4 | anonymous hidden | min | yes | 1740 | 9 | 1 | - | 1325 | 1452 | yes |  |
| 1x4 | anonymous hidden | full | yes | 2207 | 15 | 2 | 1 | 1516 | 1601 | yes |  |
| 1x4 | anonymous hidden | full+spl | yes | 2575 | 22 | 2 | - | 1648 | 1605 | yes |  |
| 1x4 | anonymous hidden | full+data | yes | 2384 | 15 | 2 | - | 1693 | 1770 | yes |  |
| 1x4 | anonymous hidden | full+reply | yes | 2500 | 15 | 2 | - | 1809 | 1892 | yes |  |
| 1x4 | anonymous hidden | full+2tree | yes | 2245 | 16 | 2 | - | 1521 | 1605 | yes |  |
| 1x4 | anonymous hidden | full+data+ring | yes | 2480 | 15 | 2 | - | 1789 | 1866 | yes |  |
| 1x4 | anonymous public owners | min | yes | 1677 | 9 | 1 | - | 1261 | 1355 | yes |  |
| 1x4 | anonymous public owners | full | yes | 2336 | 19 | 3 | 1 | 1452 | 1504 | yes |  |
| 1x4 | anonymous public owners | full+spl | yes | 2704 | 26 | 3 | - | 1584 | 1508 | yes |  |
| 1x4 | anonymous public owners | full+data | yes | 2513 | 19 | 3 | - | 1629 | 1673 | yes |  |
| 1x4 | anonymous public owners | full+reply | yes | 2629 | 19 | 3 | - | 1745 | 1795 | yes |  |
| 1x4 | anonymous public owners | full+2tree | yes | 2374 | 20 | 3 | - | 1457 | 1508 | yes |  |
| 1x4 | anonymous public owners | full+data+ring | yes | 2609 | 19 | 3 | - | 1725 | 1769 | yes |  |
| 1x4 | anonymous public owners | full+data+program | yes | 2480 | 16 | 2 | - | 1593 | 1637 | yes |  |
| 1x4 | anonymous public owners | full+data+ring+program | yes | 2576 | 16 | 2 | - | 1689 | 1733 | yes |  |
| 1x4 | confidential entry | min | yes | 1757 | 9 | 1 | - | 1341 | 1427 | yes |  |
| 1x4 | confidential entry | full | yes | 2416 | 19 | 3 | 1 | 1532 | 1576 | yes |  |
| 1x4 | confidential entry | full+spl | yes | 2784 | 26 | 3 | - | 1664 | 1580 | yes |  |
| 1x4 | confidential entry | full+data | yes | 2593 | 19 | 3 | - | 1709 | 1745 | yes |  |
| 1x4 | confidential entry | full+reply | yes | 2416 | 19 | 3 | - | 1532 | 1576 | yes |  |
| 1x4 | confidential entry | full+2tree | yes | 2454 | 20 | 3 | - | 1537 | 1580 | yes |  |
| 1x4 | confidential entry | full+data+ring | yes | 2689 | 19 | 3 | - | 1805 | 1841 | yes |  |
| 1x4 | anonymous entry | min | yes | 1916 | 9 | 1 | - | 1501 | 1596 | yes |  |
| 1x4 | anonymous entry | full | yes | 2383 | 15 | 2 | 1 | 1692 | 1745 | yes |  |
| 1x4 | anonymous entry | full+spl | yes | 2751 | 22 | 2 | - | 1824 | 1749 | yes |  |
| 1x4 | anonymous entry | full+data | yes | 2560 | 15 | 2 | - | 1869 | 1914 | yes |  |
| 1x4 | anonymous entry | full+reply | yes | 2676 | 15 | 2 | - | 1985 | 2036 | yes |  |
| 1x4 | anonymous entry | full+2tree | yes | 2421 | 16 | 2 | - | 1697 | 1749 | yes |  |
| 1x4 | anonymous entry | full+data+ring | yes | 2656 | 15 | 2 | - | 1965 | 2010 | yes |  |
| 1x4 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 1x4 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 1x8 | confidential signer | min | yes | 2321 | 9 | 1 | - | 1905 | 2135 | yes |  |
| 1x8 | confidential signer | full | yes | 2980 | 19 | 3 | 1 | 2096 | 2284 | yes |  |
| 1x8 | confidential signer | full+spl | yes | 3348 | 26 | 3 | - | 2228 | 2288 | yes |  |
| 1x8 | confidential signer | full+data | yes | 3301 | 19 | 3 | - | 2417 | 2589 | yes |  |
| 1x8 | confidential signer | full+reply | yes | 2980 | 19 | 3 | - | 2096 | 2284 | yes |  |
| 1x8 | confidential signer | full+2tree | yes | 3018 | 20 | 3 | - | 2101 | 2288 | yes |  |
| 1x8 | confidential signer | full+data+ring | yes | 3525 | 19 | 3 | - | 2641 | 2813 | yes |  |
| 1x8 | confidential signer | full+data+program | yes | 3268 | 16 | 2 | - | 2381 | 2553 | yes |  |
| 1x8 | confidential signer | full+data+ring+program | yes | 3492 | 16 | 2 | - | 2605 | 2777 | yes |  |
| 1x8 | confidential in-proof | min | yes | 2416 | 9 | 1 | - | 2001 | 2232 | yes |  |
| 1x8 | confidential in-proof | full | yes | 2979 | 18 | 2 | 1 | 2192 | 2381 | yes |  |
| 1x8 | confidential in-proof | full+spl | yes | 3347 | 25 | 2 | - | 2324 | 2385 | yes |  |
| 1x8 | confidential in-proof | full+data | yes | 3300 | 18 | 2 | - | 2513 | 2686 | yes |  |
| 1x8 | confidential in-proof | full+reply | yes | 2979 | 18 | 2 | - | 2192 | 2381 | yes |  |
| 1x8 | confidential in-proof | full+2tree | yes | 3017 | 19 | 2 | - | 2197 | 2385 | yes |  |
| 1x8 | confidential in-proof | full+data+ring | yes | 3524 | 18 | 2 | - | 2737 | 2910 | yes |  |
| 1x8 | anonymous hidden | min | yes | 2448 | 9 | 1 | - | 2033 | 2304 | yes |  |
| 1x8 | anonymous hidden | full | yes | 2915 | 15 | 2 | 1 | 2224 | 2453 | yes |  |
| 1x8 | anonymous hidden | full+spl | yes | 3283 | 22 | 2 | - | 2356 | 2457 | yes |  |
| 1x8 | anonymous hidden | full+data | yes | 3236 | 15 | 2 | - | 2545 | 2758 | yes |  |
| 1x8 | anonymous hidden | full+reply | yes | 3596 | 15 | 2 | - | 2905 | 3132 | yes |  |
| 1x8 | anonymous hidden | full+2tree | yes | 2953 | 16 | 2 | - | 2229 | 2457 | yes |  |
| 1x8 | anonymous hidden | full+data+ring | yes | 3460 | 15 | 2 | - | 2769 | 2982 | yes |  |
| 1x8 | anonymous public owners | min | yes | 2385 | 9 | 1 | - | 1969 | 2207 | yes |  |
| 1x8 | anonymous public owners | full | yes | 3044 | 19 | 3 | 1 | 2160 | 2356 | yes |  |
| 1x8 | anonymous public owners | full+spl | yes | 3412 | 26 | 3 | - | 2292 | 2360 | yes |  |
| 1x8 | anonymous public owners | full+data | yes | 3365 | 19 | 3 | - | 2481 | 2661 | yes |  |
| 1x8 | anonymous public owners | full+reply | yes | 3725 | 19 | 3 | - | 2841 | 3035 | yes |  |
| 1x8 | anonymous public owners | full+2tree | yes | 3082 | 20 | 3 | - | 2165 | 2360 | yes |  |
| 1x8 | anonymous public owners | full+data+ring | yes | 3589 | 19 | 3 | - | 2705 | 2885 | yes |  |
| 1x8 | anonymous public owners | full+data+program | yes | 3332 | 16 | 2 | - | 2445 | 2625 | yes |  |
| 1x8 | anonymous public owners | full+data+ring+program | yes | 3556 | 16 | 2 | - | 2669 | 2849 | yes |  |
| 1x8 | confidential entry | min | yes | 2465 | 9 | 1 | - | 2049 | 2279 | yes |  |
| 1x8 | confidential entry | full | yes | 3124 | 19 | 3 | 1 | 2240 | 2428 | yes |  |
| 1x8 | confidential entry | full+spl | yes | 3492 | 26 | 3 | - | 2372 | 2432 | yes |  |
| 1x8 | confidential entry | full+data | yes | 3445 | 19 | 3 | - | 2561 | 2733 | yes |  |
| 1x8 | confidential entry | full+reply | yes | 3124 | 19 | 3 | - | 2240 | 2428 | yes |  |
| 1x8 | confidential entry | full+2tree | yes | 3162 | 20 | 3 | - | 2245 | 2432 | yes |  |
| 1x8 | confidential entry | full+data+ring | yes | 3669 | 19 | 3 | - | 2785 | 2957 | yes |  |
| 1x8 | anonymous entry | min | yes | 2624 | 9 | 1 | - | 2209 | 2448 | yes |  |
| 1x8 | anonymous entry | full | yes | 3091 | 15 | 2 | 1 | 2400 | 2597 | yes |  |
| 1x8 | anonymous entry | full+spl | yes | 3459 | 22 | 2 | - | 2532 | 2601 | yes |  |
| 1x8 | anonymous entry | full+data | yes | 3412 | 15 | 2 | - | 2721 | 2902 | yes |  |
| 1x8 | anonymous entry | full+reply | yes | 3772 | 15 | 2 | - | 3081 | 3276 | yes |  |
| 1x8 | anonymous entry | full+2tree | yes | 3129 | 16 | 2 | - | 2405 | 2601 | yes |  |
| 1x8 | anonymous entry | full+data+ring | yes | 3636 | 15 | 2 | - | 2945 | 3126 | yes |  |
| 1x8 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 1x8 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 2x2 | confidential signer | min | yes | 1324 | 10 | 1 | - | 875 | 890 | yes |  |
| 2x2 | confidential signer | full | yes | 1983 | 20 | 3 | 2 | 1066 | 1039 | yes |  |
| 2x2 | confidential signer | full+spl | yes | 2351 | 27 | 3 | - | 1198 | 1043 | yes |  |
| 2x2 | confidential signer | full+data | yes | 2088 | 20 | 3 | - | 1171 | 1140 | yes |  |
| 2x2 | confidential signer | full+reply | yes | 1983 | 20 | 3 | - | 1066 | 1039 | yes |  |
| 2x2 | confidential signer | full+2tree | yes | 2021 | 21 | 3 | - | 1071 | 1043 | yes |  |
| 2x2 | confidential signer | full+data+ring | yes | 2120 | 20 | 3 | - | 1203 | 1172 | yes |  |
| 2x2 | confidential signer | full+data+program | yes | 2055 | 17 | 2 | - | 1135 | 1104 | yes |  |
| 2x2 | confidential signer | full+data+ring+program | yes | 2087 | 17 | 2 | - | 1167 | 1136 | yes |  |
| 2x2 | confidential in-proof | min | yes | 1419 | 10 | 1 | - | 971 | 987 | yes |  |
| 2x2 | confidential in-proof | full | yes | 1982 | 19 | 2 | 2 | 1162 | 1136 | yes |  |
| 2x2 | confidential in-proof | full+spl | yes | 2350 | 26 | 2 | - | 1294 | 1140 | yes |  |
| 2x2 | confidential in-proof | full+data | yes | 2087 | 19 | 2 | - | 1267 | 1237 | yes |  |
| 2x2 | confidential in-proof | full+reply | yes | 1982 | 19 | 2 | - | 1162 | 1136 | yes |  |
| 2x2 | confidential in-proof | full+2tree | yes | 2020 | 20 | 2 | - | 1167 | 1140 | yes |  |
| 2x2 | confidential in-proof | full+data+ring | yes | 2119 | 19 | 2 | - | 1299 | 1269 | yes |  |
| 2x2 | anonymous hidden | min | yes | 1451 | 10 | 1 | - | 1003 | 1059 | yes |  |
| 2x2 | anonymous hidden | full | yes | 1918 | 16 | 2 | 2 | 1194 | 1208 | yes |  |
| 2x2 | anonymous hidden | full+spl | yes | 2286 | 23 | 2 | - | 1326 | 1212 | yes |  |
| 2x2 | anonymous hidden | full+data | yes | 2023 | 16 | 2 | - | 1299 | 1309 | yes |  |
| 2x2 | anonymous hidden | full+reply | yes | 2017 | 16 | 2 | - | 1293 | 1305 | yes |  |
| 2x2 | anonymous hidden | full+2tree | yes | 1956 | 17 | 2 | - | 1199 | 1212 | yes |  |
| 2x2 | anonymous hidden | full+data+ring | yes | 2055 | 16 | 2 | - | 1331 | 1341 | yes |  |
| 2x2 | anonymous public owners | min | yes | 1388 | 10 | 1 | - | 939 | 962 | yes |  |
| 2x2 | anonymous public owners | full | yes | 2047 | 20 | 3 | 2 | 1130 | 1111 | yes |  |
| 2x2 | anonymous public owners | full+spl | yes | 2415 | 27 | 3 | - | 1262 | 1115 | yes |  |
| 2x2 | anonymous public owners | full+data | yes | 2152 | 20 | 3 | - | 1235 | 1212 | yes |  |
| 2x2 | anonymous public owners | full+reply | yes | 2146 | 20 | 3 | - | 1229 | 1208 | yes |  |
| 2x2 | anonymous public owners | full+2tree | yes | 2085 | 21 | 3 | - | 1135 | 1115 | yes |  |
| 2x2 | anonymous public owners | full+data+ring | yes | 2184 | 20 | 3 | - | 1267 | 1244 | yes |  |
| 2x2 | anonymous public owners | full+data+program | yes | 2119 | 17 | 2 | - | 1199 | 1176 | yes |  |
| 2x2 | anonymous public owners | full+data+ring+program | yes | 2151 | 17 | 2 | - | 1231 | 1208 | yes |  |
| 2x2 | confidential entry | min | yes | 1604 | 10 | 1 | - | 1155 | 1170 | yes |  |
| 2x2 | confidential entry | full | yes | 2263 | 20 | 3 | 2 | 1346 | 1319 | yes |  |
| 2x2 | confidential entry | full+spl | yes | 2631 | 27 | 3 | - | 1478 | 1323 | yes |  |
| 2x2 | confidential entry | full+data | yes | 2368 | 20 | 3 | - | 1451 | 1420 | yes |  |
| 2x2 | confidential entry | full+reply | yes | 2263 | 20 | 3 | - | 1346 | 1319 | yes |  |
| 2x2 | confidential entry | full+2tree | yes | 2301 | 21 | 3 | - | 1351 | 1323 | yes |  |
| 2x2 | confidential entry | full+data+ring | yes | 2400 | 20 | 3 | - | 1483 | 1452 | yes |  |
| 2x2 | anonymous entry | min | yes | 1763 | 10 | 1 | - | 1315 | 1339 | yes |  |
| 2x2 | anonymous entry | full | yes | 2230 | 16 | 2 | 2 | 1506 | 1488 | yes |  |
| 2x2 | anonymous entry | full+spl | yes | 2598 | 23 | 2 | - | 1638 | 1492 | yes |  |
| 2x2 | anonymous entry | full+data | yes | 2335 | 16 | 2 | - | 1611 | 1589 | yes |  |
| 2x2 | anonymous entry | full+reply | yes | 2329 | 16 | 2 | - | 1605 | 1585 | yes |  |
| 2x2 | anonymous entry | full+2tree | yes | 2268 | 17 | 2 | - | 1511 | 1492 | yes |  |
| 2x2 | anonymous entry | full+data+ring | yes | 2367 | 16 | 2 | - | 1643 | 1621 | yes |  |
| 2x2 | delegate | min | yes | 1324 | 10 | 1 | - | 875 | 890 | yes |  |
| 2x2 | delegate | full | yes | 1771 | 16 | 2 | - | 1050 | 1039 | yes |  |
| 2x2 | delegate | full+spl | yes | 2011 | 23 | 2 | - | 1054 | 1043 | yes |  |
| 2x2 | delegate | full+data | yes | 1876 | 16 | 2 | - | 1155 | 1140 | yes |  |
| 2x2 | delegate | full+reply | yes | 1771 | 16 | 2 | - | 1050 | 1039 | yes |  |
| 2x2 | delegate | full+2tree | yes | 1809 | 17 | 2 | - | 1055 | 1043 | yes |  |
| 2x2 | delegate | full+data+ring | yes | 1908 | 16 | 2 | - | 1187 | 1172 | yes |  |
| 2x2 | anonymous delegate | min | yes | 1388 | 10 | 1 | - | 939 | 962 | yes |  |
| 2x2 | anonymous delegate | full | yes | 1835 | 16 | 2 | - | 1114 | 1111 | yes |  |
| 2x2 | anonymous delegate | full+spl | yes | 2075 | 23 | 2 | - | 1118 | 1115 | yes |  |
| 2x2 | anonymous delegate | full+data | yes | 1940 | 16 | 2 | - | 1219 | 1212 | yes |  |
| 2x2 | anonymous delegate | full+reply | yes | 1934 | 16 | 2 | - | 1213 | 1208 | yes |  |
| 2x2 | anonymous delegate | full+2tree | yes | 1873 | 17 | 2 | - | 1119 | 1115 | yes |  |
| 2x2 | anonymous delegate | full+data+ring | yes | 1972 | 16 | 2 | - | 1251 | 1244 | yes |  |
| 2x4 | confidential signer | min | yes | 1678 | 10 | 1 | - | 1229 | 1316 | yes |  |
| 2x4 | confidential signer | full | yes | 2337 | 20 | 3 | 2 | 1420 | 1465 | yes |  |
| 2x4 | confidential signer | full+spl | yes | 2705 | 27 | 3 | - | 1552 | 1469 | yes |  |
| 2x4 | confidential signer | full+data | yes | 2514 | 20 | 3 | - | 1597 | 1634 | yes |  |
| 2x4 | confidential signer | full+reply | yes | 2337 | 20 | 3 | - | 1420 | 1465 | yes |  |
| 2x4 | confidential signer | full+2tree | yes | 2375 | 21 | 3 | - | 1425 | 1469 | yes |  |
| 2x4 | confidential signer | full+data+ring | yes | 2610 | 20 | 3 | - | 1693 | 1730 | yes |  |
| 2x4 | confidential signer | full+data+program | yes | 2481 | 17 | 2 | - | 1561 | 1598 | yes |  |
| 2x4 | confidential signer | full+data+ring+program | yes | 2577 | 17 | 2 | - | 1657 | 1694 | yes |  |
| 2x4 | confidential in-proof | min | yes | 1773 | 10 | 1 | - | 1325 | 1413 | yes |  |
| 2x4 | confidential in-proof | full | yes | 2336 | 19 | 2 | 2 | 1516 | 1562 | yes |  |
| 2x4 | confidential in-proof | full+spl | yes | 2704 | 26 | 2 | - | 1648 | 1566 | yes |  |
| 2x4 | confidential in-proof | full+data | yes | 2513 | 19 | 2 | - | 1693 | 1731 | yes |  |
| 2x4 | confidential in-proof | full+reply | yes | 2336 | 19 | 2 | - | 1516 | 1562 | yes |  |
| 2x4 | confidential in-proof | full+2tree | yes | 2374 | 20 | 2 | - | 1521 | 1566 | yes |  |
| 2x4 | confidential in-proof | full+data+ring | yes | 2609 | 19 | 2 | - | 1789 | 1827 | yes |  |
| 2x4 | anonymous hidden | min | yes | 1805 | 10 | 1 | - | 1357 | 1485 | yes |  |
| 2x4 | anonymous hidden | full | yes | 2272 | 16 | 2 | 2 | 1548 | 1634 | yes |  |
| 2x4 | anonymous hidden | full+spl | yes | 2640 | 23 | 2 | - | 1680 | 1638 | yes |  |
| 2x4 | anonymous hidden | full+data | yes | 2449 | 16 | 2 | - | 1725 | 1803 | yes |  |
| 2x4 | anonymous hidden | full+reply | yes | 2565 | 16 | 2 | - | 1841 | 1925 | yes |  |
| 2x4 | anonymous hidden | full+2tree | yes | 2310 | 17 | 2 | - | 1553 | 1638 | yes |  |
| 2x4 | anonymous hidden | full+data+ring | yes | 2545 | 16 | 2 | - | 1821 | 1899 | yes |  |
| 2x4 | anonymous public owners | min | yes | 1742 | 10 | 1 | - | 1293 | 1388 | yes |  |
| 2x4 | anonymous public owners | full | yes | 2401 | 20 | 3 | 2 | 1484 | 1537 | yes |  |
| 2x4 | anonymous public owners | full+spl | yes | 2769 | 27 | 3 | - | 1616 | 1541 | yes |  |
| 2x4 | anonymous public owners | full+data | yes | 2578 | 20 | 3 | - | 1661 | 1706 | yes |  |
| 2x4 | anonymous public owners | full+reply | yes | 2694 | 20 | 3 | - | 1777 | 1828 | yes |  |
| 2x4 | anonymous public owners | full+2tree | yes | 2439 | 21 | 3 | - | 1489 | 1541 | yes |  |
| 2x4 | anonymous public owners | full+data+ring | yes | 2674 | 20 | 3 | - | 1757 | 1802 | yes |  |
| 2x4 | anonymous public owners | full+data+program | yes | 2545 | 17 | 2 | - | 1625 | 1670 | yes |  |
| 2x4 | anonymous public owners | full+data+ring+program | yes | 2641 | 17 | 2 | - | 1721 | 1766 | yes |  |
| 2x4 | confidential entry | min | yes | 1958 | 10 | 1 | - | 1509 | 1596 | yes |  |
| 2x4 | confidential entry | full | yes | 2617 | 20 | 3 | 2 | 1700 | 1745 | yes |  |
| 2x4 | confidential entry | full+spl | yes | 2985 | 27 | 3 | - | 1832 | 1749 | yes |  |
| 2x4 | confidential entry | full+data | yes | 2794 | 20 | 3 | - | 1877 | 1914 | yes |  |
| 2x4 | confidential entry | full+reply | yes | 2617 | 20 | 3 | - | 1700 | 1745 | yes |  |
| 2x4 | confidential entry | full+2tree | yes | 2655 | 21 | 3 | - | 1705 | 1749 | yes |  |
| 2x4 | confidential entry | full+data+ring | yes | 2890 | 20 | 3 | - | 1973 | 2010 | yes |  |
| 2x4 | anonymous entry | min | yes | 2117 | 10 | 1 | - | 1669 | 1765 | yes |  |
| 2x4 | anonymous entry | full | yes | 2584 | 16 | 2 | 2 | 1860 | 1914 | yes |  |
| 2x4 | anonymous entry | full+spl | yes | 2952 | 23 | 2 | - | 1992 | 1918 | yes |  |
| 2x4 | anonymous entry | full+data | yes | 2761 | 16 | 2 | - | 2037 | 2083 | yes |  |
| 2x4 | anonymous entry | full+reply | yes | 2877 | 16 | 2 | - | 2153 | 2205 | yes |  |
| 2x4 | anonymous entry | full+2tree | yes | 2622 | 17 | 2 | - | 1865 | 1918 | yes |  |
| 2x4 | anonymous entry | full+data+ring | yes | 2857 | 16 | 2 | - | 2133 | 2179 | yes |  |
| 2x4 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 2x4 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 1x16 | confidential signer | min | yes | 3738 | 9 | 1 | - | 3322 | 3839 | yes |  |
| 1x16 | confidential signer | full | yes | 4397 | 19 | 3 | none | 3513 | 3988 | no | 301 B |
| 1x16 | confidential signer | full+spl | yes | 4765 | 26 | 3 | - | 3645 | 3992 | no | 669 B |
| 1x16 | confidential signer | full+data | yes | 5006 | 19 | 3 | - | 4122 | 4565 | no | 910 B |
| 1x16 | confidential signer | full+reply | yes | 4397 | 19 | 3 | - | 3513 | 3988 | no | 301 B |
| 1x16 | confidential signer | full+2tree | yes | 4435 | 20 | 3 | - | 3518 | 3992 | no | 339 B |
| 1x16 | confidential signer | full+data+ring | yes | 5486 | 19 | 3 | - | 4602 | 5045 | no | 1390 B |
| 1x16 | confidential signer | full+data+program | yes | 4973 | 16 | 2 | - | 4086 | 4529 | no | 877 B |
| 1x16 | confidential signer | full+data+ring+program | yes | 5453 | 16 | 2 | - | 4566 | 5009 | no | 1357 B |
| 1x16 | confidential in-proof | min | yes | 3833 | 9 | 1 | - | 3418 | 3936 | yes |  |
| 1x16 | confidential in-proof | full | yes | 4396 | 18 | 2 | none | 3609 | 4085 | no | 300 B |
| 1x16 | confidential in-proof | full+spl | yes | 4764 | 25 | 2 | - | 3741 | 4089 | no | 668 B |
| 1x16 | confidential in-proof | full+data | yes | 5005 | 18 | 2 | - | 4218 | 4662 | no | 909 B |
| 1x16 | confidential in-proof | full+reply | yes | 4396 | 18 | 2 | - | 3609 | 4085 | no | 300 B |
| 1x16 | confidential in-proof | full+2tree | yes | 4434 | 19 | 2 | - | 3614 | 4089 | no | 338 B |
| 1x16 | confidential in-proof | full+data+ring | yes | 5485 | 18 | 2 | - | 4698 | 5142 | no | 1389 B |
| 1x16 | anonymous hidden | min | yes | 3865 | 9 | 1 | - | 3450 | 4008 | yes |  |
| 1x16 | anonymous hidden | full | yes | 4332 | 15 | 2 | none | 3641 | 4157 | no | 236 B |
| 1x16 | anonymous hidden | full+spl | yes | 4700 | 22 | 2 | - | 3773 | 4161 | no | 604 B |
| 1x16 | anonymous hidden | full+data | yes | 4941 | 15 | 2 | - | 4250 | 4734 | no | 845 B |
| 1x16 | anonymous hidden | full+reply | yes | 5789 | 15 | 2 | - | 5098 | 5612 | no | 1693 B |
| 1x16 | anonymous hidden | full+2tree | yes | 4370 | 16 | 2 | - | 3646 | 4161 | no | 274 B |
| 1x16 | anonymous hidden | full+data+ring | yes | 5421 | 15 | 2 | - | 4730 | 5214 | no | 1325 B |
| 1x16 | anonymous public owners | min | yes | 3802 | 9 | 1 | - | 3386 | 3911 | yes |  |
| 1x16 | anonymous public owners | full | yes | 4461 | 19 | 3 | none | 3577 | 4060 | no | 365 B |
| 1x16 | anonymous public owners | full+spl | yes | 4829 | 26 | 3 | - | 3709 | 4064 | no | 733 B |
| 1x16 | anonymous public owners | full+data | yes | 5070 | 19 | 3 | - | 4186 | 4637 | no | 974 B |
| 1x16 | anonymous public owners | full+reply | yes | 5918 | 19 | 3 | - | 5034 | 5515 | no | 1822 B |
| 1x16 | anonymous public owners | full+2tree | yes | 4499 | 20 | 3 | - | 3582 | 4064 | no | 403 B |
| 1x16 | anonymous public owners | full+data+ring | yes | 5550 | 19 | 3 | - | 4666 | 5117 | no | 1454 B |
| 1x16 | anonymous public owners | full+data+program | yes | 5037 | 16 | 2 | - | 4150 | 4601 | no | 941 B |
| 1x16 | anonymous public owners | full+data+ring+program | yes | 5517 | 16 | 2 | - | 4630 | 5081 | no | 1421 B |
| 1x16 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 1x16 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 1x16 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 2x8 | confidential signer | min | yes | 2386 | 10 | 1 | - | 1937 | 2168 | yes |  |
| 2x8 | confidential signer | full | yes | 3045 | 20 | 3 | 2 | 2128 | 2317 | yes |  |
| 2x8 | confidential signer | full+spl | yes | 3413 | 27 | 3 | - | 2260 | 2321 | yes |  |
| 2x8 | confidential signer | full+data | yes | 3366 | 20 | 3 | - | 2449 | 2622 | yes |  |
| 2x8 | confidential signer | full+reply | yes | 3045 | 20 | 3 | - | 2128 | 2317 | yes |  |
| 2x8 | confidential signer | full+2tree | yes | 3083 | 21 | 3 | - | 2133 | 2321 | yes |  |
| 2x8 | confidential signer | full+data+ring | yes | 3590 | 20 | 3 | - | 2673 | 2846 | yes |  |
| 2x8 | confidential signer | full+data+program | yes | 3333 | 17 | 2 | - | 2413 | 2586 | yes |  |
| 2x8 | confidential signer | full+data+ring+program | yes | 3557 | 17 | 2 | - | 2637 | 2810 | yes |  |
| 2x8 | confidential in-proof | min | yes | 2481 | 10 | 1 | - | 2033 | 2265 | yes |  |
| 2x8 | confidential in-proof | full | yes | 3044 | 19 | 2 | 2 | 2224 | 2414 | yes |  |
| 2x8 | confidential in-proof | full+spl | yes | 3412 | 26 | 2 | - | 2356 | 2418 | yes |  |
| 2x8 | confidential in-proof | full+data | yes | 3365 | 19 | 2 | - | 2545 | 2719 | yes |  |
| 2x8 | confidential in-proof | full+reply | yes | 3044 | 19 | 2 | - | 2224 | 2414 | yes |  |
| 2x8 | confidential in-proof | full+2tree | yes | 3082 | 20 | 2 | - | 2229 | 2418 | yes |  |
| 2x8 | confidential in-proof | full+data+ring | yes | 3589 | 19 | 2 | - | 2769 | 2943 | yes |  |
| 2x8 | anonymous hidden | min | yes | 2513 | 10 | 1 | - | 2065 | 2337 | yes |  |
| 2x8 | anonymous hidden | full | yes | 2980 | 16 | 2 | 2 | 2256 | 2486 | yes |  |
| 2x8 | anonymous hidden | full+spl | yes | 3348 | 23 | 2 | - | 2388 | 2490 | yes |  |
| 2x8 | anonymous hidden | full+data | yes | 3301 | 16 | 2 | - | 2577 | 2791 | yes |  |
| 2x8 | anonymous hidden | full+reply | yes | 3661 | 16 | 2 | - | 2937 | 3165 | yes |  |
| 2x8 | anonymous hidden | full+2tree | yes | 3018 | 17 | 2 | - | 2261 | 2490 | yes |  |
| 2x8 | anonymous hidden | full+data+ring | yes | 3525 | 16 | 2 | - | 2801 | 3015 | yes |  |
| 2x8 | anonymous public owners | min | yes | 2450 | 10 | 1 | - | 2001 | 2240 | yes |  |
| 2x8 | anonymous public owners | full | yes | 3109 | 20 | 3 | 2 | 2192 | 2389 | yes |  |
| 2x8 | anonymous public owners | full+spl | yes | 3477 | 27 | 3 | - | 2324 | 2393 | yes |  |
| 2x8 | anonymous public owners | full+data | yes | 3430 | 20 | 3 | - | 2513 | 2694 | yes |  |
| 2x8 | anonymous public owners | full+reply | yes | 3790 | 20 | 3 | - | 2873 | 3068 | yes |  |
| 2x8 | anonymous public owners | full+2tree | yes | 3147 | 21 | 3 | - | 2197 | 2393 | yes |  |
| 2x8 | anonymous public owners | full+data+ring | yes | 3654 | 20 | 3 | - | 2737 | 2918 | yes |  |
| 2x8 | anonymous public owners | full+data+program | yes | 3397 | 17 | 2 | - | 2477 | 2658 | yes |  |
| 2x8 | anonymous public owners | full+data+ring+program | yes | 3621 | 17 | 2 | - | 2701 | 2882 | yes |  |
| 2x8 | confidential entry | min | yes | 2666 | 10 | 1 | - | 2217 | 2448 | yes |  |
| 2x8 | confidential entry | full | yes | 3325 | 20 | 3 | 2 | 2408 | 2597 | yes |  |
| 2x8 | confidential entry | full+spl | yes | 3693 | 27 | 3 | - | 2540 | 2601 | yes |  |
| 2x8 | confidential entry | full+data | yes | 3646 | 20 | 3 | - | 2729 | 2902 | yes |  |
| 2x8 | confidential entry | full+reply | yes | 3325 | 20 | 3 | - | 2408 | 2597 | yes |  |
| 2x8 | confidential entry | full+2tree | yes | 3363 | 21 | 3 | - | 2413 | 2601 | yes |  |
| 2x8 | confidential entry | full+data+ring | yes | 3870 | 20 | 3 | - | 2953 | 3126 | yes |  |
| 2x8 | anonymous entry | min | yes | 2825 | 10 | 1 | - | 2377 | 2617 | yes |  |
| 2x8 | anonymous entry | full | yes | 3292 | 16 | 2 | 2 | 2568 | 2766 | yes |  |
| 2x8 | anonymous entry | full+spl | yes | 3660 | 23 | 2 | - | 2700 | 2770 | yes |  |
| 2x8 | anonymous entry | full+data | yes | 3613 | 16 | 2 | - | 2889 | 3071 | yes |  |
| 2x8 | anonymous entry | full+reply | yes | 3973 | 16 | 2 | - | 3249 | 3445 | yes |  |
| 2x8 | anonymous entry | full+2tree | yes | 3330 | 17 | 2 | - | 2573 | 2770 | yes |  |
| 2x8 | anonymous entry | full+data+ring | yes | 3837 | 16 | 2 | - | 3113 | 3295 | yes |  |
| 2x8 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 2x8 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 3x2 | confidential signer | min | yes | 1389 | 11 | 1 | - | 907 | 923 | yes |  |
| 3x2 | confidential signer | full | yes | 2048 | 21 | 3 | 3 | 1098 | 1072 | yes |  |
| 3x2 | confidential signer | full+spl | yes | 2416 | 28 | 3 | - | 1230 | 1076 | yes |  |
| 3x2 | confidential signer | full+data | yes | 2153 | 21 | 3 | - | 1203 | 1173 | yes |  |
| 3x2 | confidential signer | full+reply | yes | 2048 | 21 | 3 | - | 1098 | 1072 | yes |  |
| 3x2 | confidential signer | full+2tree | yes | 2086 | 22 | 3 | - | 1103 | 1076 | yes |  |
| 3x2 | confidential signer | full+data+ring | yes | 2185 | 21 | 3 | - | 1235 | 1205 | yes |  |
| 3x2 | confidential signer | full+data+program | yes | 2120 | 18 | 2 | - | 1167 | 1137 | yes |  |
| 3x2 | confidential signer | full+data+ring+program | yes | 2152 | 18 | 2 | - | 1199 | 1169 | yes |  |
| 3x2 | confidential in-proof | min | yes | 1484 | 11 | 1 | - | 1003 | 1020 | yes |  |
| 3x2 | confidential in-proof | full | yes | 2047 | 20 | 2 | 3 | 1194 | 1169 | yes |  |
| 3x2 | confidential in-proof | full+spl | yes | 2415 | 27 | 2 | - | 1326 | 1173 | yes |  |
| 3x2 | confidential in-proof | full+data | yes | 2152 | 20 | 2 | - | 1299 | 1270 | yes |  |
| 3x2 | confidential in-proof | full+reply | yes | 2047 | 20 | 2 | - | 1194 | 1169 | yes |  |
| 3x2 | confidential in-proof | full+2tree | yes | 2085 | 21 | 2 | - | 1199 | 1173 | yes |  |
| 3x2 | confidential in-proof | full+data+ring | yes | 2184 | 20 | 2 | - | 1331 | 1302 | yes |  |
| 3x2 | anonymous hidden | min | yes | 1516 | 11 | 1 | - | 1035 | 1092 | yes |  |
| 3x2 | anonymous hidden | full | yes | 1983 | 17 | 2 | 3 | 1226 | 1241 | yes |  |
| 3x2 | anonymous hidden | full+spl | yes | 2351 | 24 | 2 | - | 1358 | 1245 | yes |  |
| 3x2 | anonymous hidden | full+data | yes | 2088 | 17 | 2 | - | 1331 | 1342 | yes |  |
| 3x2 | anonymous hidden | full+reply | yes | 2082 | 17 | 2 | - | 1325 | 1338 | yes |  |
| 3x2 | anonymous hidden | full+2tree | yes | 2021 | 18 | 2 | - | 1231 | 1245 | yes |  |
| 3x2 | anonymous hidden | full+data+ring | yes | 2120 | 17 | 2 | - | 1363 | 1374 | yes |  |
| 3x2 | anonymous public owners | min | yes | 1453 | 11 | 1 | - | 971 | 995 | yes |  |
| 3x2 | anonymous public owners | full | yes | 2112 | 21 | 3 | 3 | 1162 | 1144 | yes |  |
| 3x2 | anonymous public owners | full+spl | yes | 2480 | 28 | 3 | - | 1294 | 1148 | yes |  |
| 3x2 | anonymous public owners | full+data | yes | 2217 | 21 | 3 | - | 1267 | 1245 | yes |  |
| 3x2 | anonymous public owners | full+reply | yes | 2211 | 21 | 3 | - | 1261 | 1241 | yes |  |
| 3x2 | anonymous public owners | full+2tree | yes | 2150 | 22 | 3 | - | 1167 | 1148 | yes |  |
| 3x2 | anonymous public owners | full+data+ring | yes | 2249 | 21 | 3 | - | 1299 | 1277 | yes |  |
| 3x2 | anonymous public owners | full+data+program | yes | 2184 | 18 | 2 | - | 1231 | 1209 | yes |  |
| 3x2 | anonymous public owners | full+data+ring+program | yes | 2216 | 18 | 2 | - | 1263 | 1241 | yes |  |
| 3x2 | confidential entry | min | yes | 1805 | 11 | 1 | - | 1323 | 1339 | yes |  |
| 3x2 | confidential entry | full | yes | 2464 | 21 | 3 | 3 | 1514 | 1488 | yes |  |
| 3x2 | confidential entry | full+spl | yes | 2832 | 28 | 3 | - | 1646 | 1492 | yes |  |
| 3x2 | confidential entry | full+data | yes | 2569 | 21 | 3 | - | 1619 | 1589 | yes |  |
| 3x2 | confidential entry | full+reply | yes | 2464 | 21 | 3 | - | 1514 | 1488 | yes |  |
| 3x2 | confidential entry | full+2tree | yes | 2502 | 22 | 3 | - | 1519 | 1492 | yes |  |
| 3x2 | confidential entry | full+data+ring | yes | 2601 | 21 | 3 | - | 1651 | 1621 | yes |  |
| 3x2 | anonymous entry | min | yes | 1964 | 11 | 1 | - | 1483 | 1508 | yes |  |
| 3x2 | anonymous entry | full | yes | 2431 | 17 | 2 | 3 | 1674 | 1657 | yes |  |
| 3x2 | anonymous entry | full+spl | yes | 2799 | 24 | 2 | - | 1806 | 1661 | yes |  |
| 3x2 | anonymous entry | full+data | yes | 2536 | 17 | 2 | - | 1779 | 1758 | yes |  |
| 3x2 | anonymous entry | full+reply | yes | 2530 | 17 | 2 | - | 1773 | 1754 | yes |  |
| 3x2 | anonymous entry | full+2tree | yes | 2469 | 18 | 2 | - | 1679 | 1661 | yes |  |
| 3x2 | anonymous entry | full+data+ring | yes | 2568 | 17 | 2 | - | 1811 | 1790 | yes |  |
| 3x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 3x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 3x4 | confidential signer | min | yes | 1743 | 11 | 1 | - | 1261 | 1349 | yes |  |
| 3x4 | confidential signer | full | yes | 2402 | 21 | 3 | 3 | 1452 | 1498 | yes |  |
| 3x4 | confidential signer | full+spl | yes | 2770 | 28 | 3 | - | 1584 | 1502 | yes |  |
| 3x4 | confidential signer | full+data | yes | 2579 | 21 | 3 | - | 1629 | 1667 | yes |  |
| 3x4 | confidential signer | full+reply | yes | 2402 | 21 | 3 | - | 1452 | 1498 | yes |  |
| 3x4 | confidential signer | full+2tree | yes | 2440 | 22 | 3 | - | 1457 | 1502 | yes |  |
| 3x4 | confidential signer | full+data+ring | yes | 2675 | 21 | 3 | - | 1725 | 1763 | yes |  |
| 3x4 | confidential signer | full+data+program | yes | 2546 | 18 | 2 | - | 1593 | 1631 | yes |  |
| 3x4 | confidential signer | full+data+ring+program | yes | 2642 | 18 | 2 | - | 1689 | 1727 | yes |  |
| 3x4 | confidential in-proof | min | yes | 1838 | 11 | 1 | - | 1357 | 1446 | yes |  |
| 3x4 | confidential in-proof | full | yes | 2401 | 20 | 2 | 3 | 1548 | 1595 | yes |  |
| 3x4 | confidential in-proof | full+spl | yes | 2769 | 27 | 2 | - | 1680 | 1599 | yes |  |
| 3x4 | confidential in-proof | full+data | yes | 2578 | 20 | 2 | - | 1725 | 1764 | yes |  |
| 3x4 | confidential in-proof | full+reply | yes | 2401 | 20 | 2 | - | 1548 | 1595 | yes |  |
| 3x4 | confidential in-proof | full+2tree | yes | 2439 | 21 | 2 | - | 1553 | 1599 | yes |  |
| 3x4 | confidential in-proof | full+data+ring | yes | 2674 | 20 | 2 | - | 1821 | 1860 | yes |  |
| 3x4 | anonymous hidden | min | yes | 1870 | 11 | 1 | - | 1389 | 1518 | yes |  |
| 3x4 | anonymous hidden | full | yes | 2337 | 17 | 2 | 3 | 1580 | 1667 | yes |  |
| 3x4 | anonymous hidden | full+spl | yes | 2705 | 24 | 2 | - | 1712 | 1671 | yes |  |
| 3x4 | anonymous hidden | full+data | yes | 2514 | 17 | 2 | - | 1757 | 1836 | yes |  |
| 3x4 | anonymous hidden | full+reply | yes | 2630 | 17 | 2 | - | 1873 | 1958 | yes |  |
| 3x4 | anonymous hidden | full+2tree | yes | 2375 | 18 | 2 | - | 1585 | 1671 | yes |  |
| 3x4 | anonymous hidden | full+data+ring | yes | 2610 | 17 | 2 | - | 1853 | 1932 | yes |  |
| 3x4 | anonymous public owners | min | yes | 1807 | 11 | 1 | - | 1325 | 1421 | yes |  |
| 3x4 | anonymous public owners | full | yes | 2466 | 21 | 3 | 3 | 1516 | 1570 | yes |  |
| 3x4 | anonymous public owners | full+spl | yes | 2834 | 28 | 3 | - | 1648 | 1574 | yes |  |
| 3x4 | anonymous public owners | full+data | yes | 2643 | 21 | 3 | - | 1693 | 1739 | yes |  |
| 3x4 | anonymous public owners | full+reply | yes | 2759 | 21 | 3 | - | 1809 | 1861 | yes |  |
| 3x4 | anonymous public owners | full+2tree | yes | 2504 | 22 | 3 | - | 1521 | 1574 | yes |  |
| 3x4 | anonymous public owners | full+data+ring | yes | 2739 | 21 | 3 | - | 1789 | 1835 | yes |  |
| 3x4 | anonymous public owners | full+data+program | yes | 2610 | 18 | 2 | - | 1657 | 1703 | yes |  |
| 3x4 | anonymous public owners | full+data+ring+program | yes | 2706 | 18 | 2 | - | 1753 | 1799 | yes |  |
| 3x4 | confidential entry | min | yes | 2159 | 11 | 1 | - | 1677 | 1765 | yes |  |
| 3x4 | confidential entry | full | yes | 2818 | 21 | 3 | 3 | 1868 | 1914 | yes |  |
| 3x4 | confidential entry | full+spl | yes | 3186 | 28 | 3 | - | 2000 | 1918 | yes |  |
| 3x4 | confidential entry | full+data | yes | 2995 | 21 | 3 | - | 2045 | 2083 | yes |  |
| 3x4 | confidential entry | full+reply | yes | 2818 | 21 | 3 | - | 1868 | 1914 | yes |  |
| 3x4 | confidential entry | full+2tree | yes | 2856 | 22 | 3 | - | 1873 | 1918 | yes |  |
| 3x4 | confidential entry | full+data+ring | yes | 3091 | 21 | 3 | - | 2141 | 2179 | yes |  |
| 3x4 | anonymous entry | min | yes | 2318 | 11 | 1 | - | 1837 | 1934 | yes |  |
| 3x4 | anonymous entry | full | yes | 2785 | 17 | 2 | 3 | 2028 | 2083 | yes |  |
| 3x4 | anonymous entry | full+spl | yes | 3153 | 24 | 2 | - | 2160 | 2087 | yes |  |
| 3x4 | anonymous entry | full+data | yes | 2962 | 17 | 2 | - | 2205 | 2252 | yes |  |
| 3x4 | anonymous entry | full+reply | yes | 3078 | 17 | 2 | - | 2321 | 2374 | yes |  |
| 3x4 | anonymous entry | full+2tree | yes | 2823 | 18 | 2 | - | 2033 | 2087 | yes |  |
| 3x4 | anonymous entry | full+data+ring | yes | 3058 | 17 | 2 | - | 2301 | 2348 | yes |  |
| 3x4 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 3x4 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 2x16 | confidential signer | min | yes | 3803 | 10 | 1 | - | 3354 | 3872 | yes |  |
| 2x16 | confidential signer | full | yes | 4462 | 20 | 3 | none | 3545 | 4021 | no | 366 B |
| 2x16 | confidential signer | full+spl | yes | 4830 | 27 | 3 | - | 3677 | 4025 | no | 734 B |
| 2x16 | confidential signer | full+data | yes | 5071 | 20 | 3 | - | 4154 | 4598 | no | 975 B |
| 2x16 | confidential signer | full+reply | yes | 4462 | 20 | 3 | - | 3545 | 4021 | no | 366 B |
| 2x16 | confidential signer | full+2tree | yes | 4500 | 21 | 3 | - | 3550 | 4025 | no | 404 B |
| 2x16 | confidential signer | full+data+ring | yes | 5551 | 20 | 3 | - | 4634 | 5078 | no | 1455 B |
| 2x16 | confidential signer | full+data+program | yes | 5038 | 17 | 2 | - | 4118 | 4562 | no | 942 B |
| 2x16 | confidential signer | full+data+ring+program | yes | 5518 | 17 | 2 | - | 4598 | 5042 | no | 1422 B |
| 2x16 | confidential in-proof | min | yes | 3898 | 10 | 1 | - | 3450 | 3969 | yes |  |
| 2x16 | confidential in-proof | full | yes | 4461 | 19 | 2 | none | 3641 | 4118 | no | 365 B |
| 2x16 | confidential in-proof | full+spl | yes | 4829 | 26 | 2 | - | 3773 | 4122 | no | 733 B |
| 2x16 | confidential in-proof | full+data | yes | 5070 | 19 | 2 | - | 4250 | 4695 | no | 974 B |
| 2x16 | confidential in-proof | full+reply | yes | 4461 | 19 | 2 | - | 3641 | 4118 | no | 365 B |
| 2x16 | confidential in-proof | full+2tree | yes | 4499 | 20 | 2 | - | 3646 | 4122 | no | 403 B |
| 2x16 | confidential in-proof | full+data+ring | yes | 5550 | 19 | 2 | - | 4730 | 5175 | no | 1454 B |
| 2x16 | anonymous hidden | min | yes | 3930 | 10 | 1 | - | 3482 | 4041 | yes |  |
| 2x16 | anonymous hidden | full | yes | 4397 | 16 | 2 | none | 3673 | 4190 | no | 301 B |
| 2x16 | anonymous hidden | full+spl | yes | 4765 | 23 | 2 | - | 3805 | 4194 | no | 669 B |
| 2x16 | anonymous hidden | full+data | yes | 5006 | 16 | 2 | - | 4282 | 4767 | no | 910 B |
| 2x16 | anonymous hidden | full+reply | yes | 5854 | 16 | 2 | - | 5130 | 5645 | no | 1758 B |
| 2x16 | anonymous hidden | full+2tree | yes | 4435 | 17 | 2 | - | 3678 | 4194 | no | 339 B |
| 2x16 | anonymous hidden | full+data+ring | yes | 5486 | 16 | 2 | - | 4762 | 5247 | no | 1390 B |
| 2x16 | anonymous public owners | min | yes | 3867 | 10 | 1 | - | 3418 | 3944 | yes |  |
| 2x16 | anonymous public owners | full | yes | 4526 | 20 | 3 | none | 3609 | 4093 | no | 430 B |
| 2x16 | anonymous public owners | full+spl | yes | 4894 | 27 | 3 | - | 3741 | 4097 | no | 798 B |
| 2x16 | anonymous public owners | full+data | yes | 5135 | 20 | 3 | - | 4218 | 4670 | no | 1039 B |
| 2x16 | anonymous public owners | full+reply | yes | 5983 | 20 | 3 | - | 5066 | 5548 | no | 1887 B |
| 2x16 | anonymous public owners | full+2tree | yes | 4564 | 21 | 3 | - | 3614 | 4097 | no | 468 B |
| 2x16 | anonymous public owners | full+data+ring | yes | 5615 | 20 | 3 | - | 4698 | 5150 | no | 1519 B |
| 2x16 | anonymous public owners | full+data+program | yes | 5102 | 17 | 2 | - | 4182 | 4634 | no | 1006 B |
| 2x16 | anonymous public owners | full+data+ring+program | yes | 5582 | 17 | 2 | - | 4662 | 5114 | no | 1486 B |
| 2x16 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 2x16 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 2x16 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 3x8 | confidential signer | min | yes | 2451 | 11 | 1 | - | 1969 | 2201 | yes |  |
| 3x8 | confidential signer | full | yes | 3110 | 21 | 3 | 3 | 2160 | 2350 | yes |  |
| 3x8 | confidential signer | full+spl | yes | 3478 | 28 | 3 | - | 2292 | 2354 | yes |  |
| 3x8 | confidential signer | full+data | yes | 3431 | 21 | 3 | - | 2481 | 2655 | yes |  |
| 3x8 | confidential signer | full+reply | yes | 3110 | 21 | 3 | - | 2160 | 2350 | yes |  |
| 3x8 | confidential signer | full+2tree | yes | 3148 | 22 | 3 | - | 2165 | 2354 | yes |  |
| 3x8 | confidential signer | full+data+ring | yes | 3655 | 21 | 3 | - | 2705 | 2879 | yes |  |
| 3x8 | confidential signer | full+data+program | yes | 3398 | 18 | 2 | - | 2445 | 2619 | yes |  |
| 3x8 | confidential signer | full+data+ring+program | yes | 3622 | 18 | 2 | - | 2669 | 2843 | yes |  |
| 3x8 | confidential in-proof | min | yes | 2546 | 11 | 1 | - | 2065 | 2298 | yes |  |
| 3x8 | confidential in-proof | full | yes | 3109 | 20 | 2 | 3 | 2256 | 2447 | yes |  |
| 3x8 | confidential in-proof | full+spl | yes | 3477 | 27 | 2 | - | 2388 | 2451 | yes |  |
| 3x8 | confidential in-proof | full+data | yes | 3430 | 20 | 2 | - | 2577 | 2752 | yes |  |
| 3x8 | confidential in-proof | full+reply | yes | 3109 | 20 | 2 | - | 2256 | 2447 | yes |  |
| 3x8 | confidential in-proof | full+2tree | yes | 3147 | 21 | 2 | - | 2261 | 2451 | yes |  |
| 3x8 | confidential in-proof | full+data+ring | yes | 3654 | 20 | 2 | - | 2801 | 2976 | yes |  |
| 3x8 | anonymous hidden | min | yes | 2578 | 11 | 1 | - | 2097 | 2370 | yes |  |
| 3x8 | anonymous hidden | full | yes | 3045 | 17 | 2 | 3 | 2288 | 2519 | yes |  |
| 3x8 | anonymous hidden | full+spl | yes | 3413 | 24 | 2 | - | 2420 | 2523 | yes |  |
| 3x8 | anonymous hidden | full+data | yes | 3366 | 17 | 2 | - | 2609 | 2824 | yes |  |
| 3x8 | anonymous hidden | full+reply | yes | 3726 | 17 | 2 | - | 2969 | 3198 | yes |  |
| 3x8 | anonymous hidden | full+2tree | yes | 3083 | 18 | 2 | - | 2293 | 2523 | yes |  |
| 3x8 | anonymous hidden | full+data+ring | yes | 3590 | 17 | 2 | - | 2833 | 3048 | yes |  |
| 3x8 | anonymous public owners | min | yes | 2515 | 11 | 1 | - | 2033 | 2273 | yes |  |
| 3x8 | anonymous public owners | full | yes | 3174 | 21 | 3 | 3 | 2224 | 2422 | yes |  |
| 3x8 | anonymous public owners | full+spl | yes | 3542 | 28 | 3 | - | 2356 | 2426 | yes |  |
| 3x8 | anonymous public owners | full+data | yes | 3495 | 21 | 3 | - | 2545 | 2727 | yes |  |
| 3x8 | anonymous public owners | full+reply | yes | 3855 | 21 | 3 | - | 2905 | 3101 | yes |  |
| 3x8 | anonymous public owners | full+2tree | yes | 3212 | 22 | 3 | - | 2229 | 2426 | yes |  |
| 3x8 | anonymous public owners | full+data+ring | yes | 3719 | 21 | 3 | - | 2769 | 2951 | yes |  |
| 3x8 | anonymous public owners | full+data+program | yes | 3462 | 18 | 2 | - | 2509 | 2691 | yes |  |
| 3x8 | anonymous public owners | full+data+ring+program | yes | 3686 | 18 | 2 | - | 2733 | 2915 | yes |  |
| 3x8 | confidential entry | min | yes | 2867 | 11 | 1 | - | 2385 | 2617 | yes |  |
| 3x8 | confidential entry | full | yes | 3526 | 21 | 3 | 3 | 2576 | 2766 | yes |  |
| 3x8 | confidential entry | full+spl | yes | 3894 | 28 | 3 | - | 2708 | 2770 | yes |  |
| 3x8 | confidential entry | full+data | yes | 3847 | 21 | 3 | - | 2897 | 3071 | yes |  |
| 3x8 | confidential entry | full+reply | yes | 3526 | 21 | 3 | - | 2576 | 2766 | yes |  |
| 3x8 | confidential entry | full+2tree | yes | 3564 | 22 | 3 | - | 2581 | 2770 | yes |  |
| 3x8 | confidential entry | full+data+ring | yes | 4071 | 21 | 3 | - | 3121 | 3295 | yes |  |
| 3x8 | anonymous entry | min | yes | 3026 | 11 | 1 | - | 2545 | 2786 | yes |  |
| 3x8 | anonymous entry | full | yes | 3493 | 17 | 2 | 3 | 2736 | 2935 | yes |  |
| 3x8 | anonymous entry | full+spl | yes | 3861 | 24 | 2 | - | 2868 | 2939 | yes |  |
| 3x8 | anonymous entry | full+data | yes | 3814 | 17 | 2 | - | 3057 | 3240 | yes |  |
| 3x8 | anonymous entry | full+reply | yes | 4174 | 17 | 2 | - | 3417 | 3614 | no | 78 B |
| 3x8 | anonymous entry | full+2tree | yes | 3531 | 18 | 2 | - | 2741 | 2939 | yes |  |
| 3x8 | anonymous entry | full+data+ring | yes | 4038 | 17 | 2 | - | 3281 | 3464 | yes |  |
| 3x8 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 3x8 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 4x2 | confidential signer | min | yes | 1454 | 12 | 1 | - | 939 | 956 | yes |  |
| 4x2 | confidential signer | full | yes | 2113 | 22 | 3 | 4 | 1130 | 1105 | yes |  |
| 4x2 | confidential signer | full+spl | yes | 2481 | 29 | 3 | - | 1262 | 1109 | yes |  |
| 4x2 | confidential signer | full+data | yes | 2218 | 22 | 3 | - | 1235 | 1206 | yes |  |
| 4x2 | confidential signer | full+reply | yes | 2113 | 22 | 3 | - | 1130 | 1105 | yes |  |
| 4x2 | confidential signer | full+2tree | yes | 2151 | 23 | 3 | - | 1135 | 1109 | yes |  |
| 4x2 | confidential signer | full+data+ring | yes | 2250 | 22 | 3 | - | 1267 | 1238 | yes |  |
| 4x2 | confidential signer | full+data+program | yes | 2185 | 19 | 2 | - | 1199 | 1170 | yes |  |
| 4x2 | confidential signer | full+data+ring+program | yes | 2217 | 19 | 2 | - | 1231 | 1202 | yes |  |
| 4x2 | confidential in-proof | min | yes | 1549 | 12 | 1 | - | 1035 | 1053 | yes |  |
| 4x2 | confidential in-proof | full | yes | 2112 | 21 | 2 | 4 | 1226 | 1202 | yes |  |
| 4x2 | confidential in-proof | full+spl | yes | 2480 | 28 | 2 | - | 1358 | 1206 | yes |  |
| 4x2 | confidential in-proof | full+data | yes | 2217 | 21 | 2 | - | 1331 | 1303 | yes |  |
| 4x2 | confidential in-proof | full+reply | yes | 2112 | 21 | 2 | - | 1226 | 1202 | yes |  |
| 4x2 | confidential in-proof | full+2tree | yes | 2150 | 22 | 2 | - | 1231 | 1206 | yes |  |
| 4x2 | confidential in-proof | full+data+ring | yes | 2249 | 21 | 2 | - | 1363 | 1335 | yes |  |
| 4x2 | anonymous hidden | min | yes | 1581 | 12 | 1 | - | 1067 | 1125 | yes |  |
| 4x2 | anonymous hidden | full | yes | 2048 | 18 | 2 | 4 | 1258 | 1274 | yes |  |
| 4x2 | anonymous hidden | full+spl | yes | 2416 | 25 | 2 | - | 1390 | 1278 | yes |  |
| 4x2 | anonymous hidden | full+data | yes | 2153 | 18 | 2 | - | 1363 | 1375 | yes |  |
| 4x2 | anonymous hidden | full+reply | yes | 2147 | 18 | 2 | - | 1357 | 1371 | yes |  |
| 4x2 | anonymous hidden | full+2tree | yes | 2086 | 19 | 2 | - | 1263 | 1278 | yes |  |
| 4x2 | anonymous hidden | full+data+ring | yes | 2185 | 18 | 2 | - | 1395 | 1407 | yes |  |
| 4x2 | anonymous public owners | min | yes | 1518 | 12 | 1 | - | 1003 | 1028 | yes |  |
| 4x2 | anonymous public owners | full | yes | 2177 | 22 | 3 | 4 | 1194 | 1177 | yes |  |
| 4x2 | anonymous public owners | full+spl | yes | 2545 | 29 | 3 | - | 1326 | 1181 | yes |  |
| 4x2 | anonymous public owners | full+data | yes | 2282 | 22 | 3 | - | 1299 | 1278 | yes |  |
| 4x2 | anonymous public owners | full+reply | yes | 2276 | 22 | 3 | - | 1293 | 1274 | yes |  |
| 4x2 | anonymous public owners | full+2tree | yes | 2215 | 23 | 3 | - | 1199 | 1181 | yes |  |
| 4x2 | anonymous public owners | full+data+ring | yes | 2314 | 22 | 3 | - | 1331 | 1310 | yes |  |
| 4x2 | anonymous public owners | full+data+program | yes | 2249 | 19 | 2 | - | 1263 | 1242 | yes |  |
| 4x2 | anonymous public owners | full+data+ring+program | yes | 2281 | 19 | 2 | - | 1295 | 1274 | yes |  |
| 4x2 | confidential entry | min | yes | 2006 | 12 | 1 | - | 1491 | 1508 | yes |  |
| 4x2 | confidential entry | full | yes | 2665 | 22 | 3 | 4 | 1682 | 1657 | yes |  |
| 4x2 | confidential entry | full+spl | yes | 3033 | 29 | 3 | - | 1814 | 1661 | yes |  |
| 4x2 | confidential entry | full+data | yes | 2770 | 22 | 3 | - | 1787 | 1758 | yes |  |
| 4x2 | confidential entry | full+reply | yes | 2665 | 22 | 3 | - | 1682 | 1657 | yes |  |
| 4x2 | confidential entry | full+2tree | yes | 2703 | 23 | 3 | - | 1687 | 1661 | yes |  |
| 4x2 | confidential entry | full+data+ring | yes | 2802 | 22 | 3 | - | 1819 | 1790 | yes |  |
| 4x2 | anonymous entry | min | yes | 2165 | 12 | 1 | - | 1651 | 1677 | yes |  |
| 4x2 | anonymous entry | full | yes | 2632 | 18 | 2 | 4 | 1842 | 1826 | yes |  |
| 4x2 | anonymous entry | full+spl | yes | 3000 | 25 | 2 | - | 1974 | 1830 | yes |  |
| 4x2 | anonymous entry | full+data | yes | 2737 | 18 | 2 | - | 1947 | 1927 | yes |  |
| 4x2 | anonymous entry | full+reply | yes | 2731 | 18 | 2 | - | 1941 | 1923 | yes |  |
| 4x2 | anonymous entry | full+2tree | yes | 2670 | 19 | 2 | - | 1847 | 1830 | yes |  |
| 4x2 | anonymous entry | full+data+ring | yes | 2769 | 18 | 2 | - | 1979 | 1959 | yes |  |
| 4x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 4x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 4x4 | confidential signer | min | yes | 1808 | 12 | 1 | - | 1293 | 1382 | yes |  |
| 4x4 | confidential signer | full | yes | 2467 | 22 | 3 | 4 | 1484 | 1531 | yes |  |
| 4x4 | confidential signer | full+spl | yes | 2835 | 29 | 3 | - | 1616 | 1535 | yes |  |
| 4x4 | confidential signer | full+data | yes | 2644 | 22 | 3 | - | 1661 | 1700 | yes |  |
| 4x4 | confidential signer | full+reply | yes | 2467 | 22 | 3 | - | 1484 | 1531 | yes |  |
| 4x4 | confidential signer | full+2tree | yes | 2505 | 23 | 3 | - | 1489 | 1535 | yes |  |
| 4x4 | confidential signer | full+data+ring | yes | 2740 | 22 | 3 | - | 1757 | 1796 | yes |  |
| 4x4 | confidential signer | full+data+program | yes | 2611 | 19 | 2 | - | 1625 | 1664 | yes |  |
| 4x4 | confidential signer | full+data+ring+program | yes | 2707 | 19 | 2 | - | 1721 | 1760 | yes |  |
| 4x4 | confidential in-proof | min | yes | 1903 | 12 | 1 | - | 1389 | 1479 | yes |  |
| 4x4 | confidential in-proof | full | yes | 2466 | 21 | 2 | 4 | 1580 | 1628 | yes |  |
| 4x4 | confidential in-proof | full+spl | yes | 2834 | 28 | 2 | - | 1712 | 1632 | yes |  |
| 4x4 | confidential in-proof | full+data | yes | 2643 | 21 | 2 | - | 1757 | 1797 | yes |  |
| 4x4 | confidential in-proof | full+reply | yes | 2466 | 21 | 2 | - | 1580 | 1628 | yes |  |
| 4x4 | confidential in-proof | full+2tree | yes | 2504 | 22 | 2 | - | 1585 | 1632 | yes |  |
| 4x4 | confidential in-proof | full+data+ring | yes | 2739 | 21 | 2 | - | 1853 | 1893 | yes |  |
| 4x4 | anonymous hidden | min | yes | 1935 | 12 | 1 | - | 1421 | 1551 | yes |  |
| 4x4 | anonymous hidden | full | yes | 2402 | 18 | 2 | 4 | 1612 | 1700 | yes |  |
| 4x4 | anonymous hidden | full+spl | yes | 2770 | 25 | 2 | - | 1744 | 1704 | yes |  |
| 4x4 | anonymous hidden | full+data | yes | 2579 | 18 | 2 | - | 1789 | 1869 | yes |  |
| 4x4 | anonymous hidden | full+reply | yes | 2695 | 18 | 2 | - | 1905 | 1991 | yes |  |
| 4x4 | anonymous hidden | full+2tree | yes | 2440 | 19 | 2 | - | 1617 | 1704 | yes |  |
| 4x4 | anonymous hidden | full+data+ring | yes | 2675 | 18 | 2 | - | 1885 | 1965 | yes |  |
| 4x4 | anonymous public owners | min | yes | 1872 | 12 | 1 | - | 1357 | 1454 | yes |  |
| 4x4 | anonymous public owners | full | yes | 2531 | 22 | 3 | 4 | 1548 | 1603 | yes |  |
| 4x4 | anonymous public owners | full+spl | yes | 2899 | 29 | 3 | - | 1680 | 1607 | yes |  |
| 4x4 | anonymous public owners | full+data | yes | 2708 | 22 | 3 | - | 1725 | 1772 | yes |  |
| 4x4 | anonymous public owners | full+reply | yes | 2824 | 22 | 3 | - | 1841 | 1894 | yes |  |
| 4x4 | anonymous public owners | full+2tree | yes | 2569 | 23 | 3 | - | 1553 | 1607 | yes |  |
| 4x4 | anonymous public owners | full+data+ring | yes | 2804 | 22 | 3 | - | 1821 | 1868 | yes |  |
| 4x4 | anonymous public owners | full+data+program | yes | 2675 | 19 | 2 | - | 1689 | 1736 | yes |  |
| 4x4 | anonymous public owners | full+data+ring+program | yes | 2771 | 19 | 2 | - | 1785 | 1832 | yes |  |
| 4x4 | confidential entry | min | yes | 2360 | 12 | 1 | - | 1845 | 1934 | yes |  |
| 4x4 | confidential entry | full | yes | 3019 | 22 | 3 | 4 | 2036 | 2083 | yes |  |
| 4x4 | confidential entry | full+spl | yes | 3387 | 29 | 3 | - | 2168 | 2087 | yes |  |
| 4x4 | confidential entry | full+data | yes | 3196 | 22 | 3 | - | 2213 | 2252 | yes |  |
| 4x4 | confidential entry | full+reply | yes | 3019 | 22 | 3 | - | 2036 | 2083 | yes |  |
| 4x4 | confidential entry | full+2tree | yes | 3057 | 23 | 3 | - | 2041 | 2087 | yes |  |
| 4x4 | confidential entry | full+data+ring | yes | 3292 | 22 | 3 | - | 2309 | 2348 | yes |  |
| 4x4 | anonymous entry | min | yes | 2519 | 12 | 1 | - | 2005 | 2103 | yes |  |
| 4x4 | anonymous entry | full | yes | 2986 | 18 | 2 | 4 | 2196 | 2252 | yes |  |
| 4x4 | anonymous entry | full+spl | yes | 3354 | 25 | 2 | - | 2328 | 2256 | yes |  |
| 4x4 | anonymous entry | full+data | yes | 3163 | 18 | 2 | - | 2373 | 2421 | yes |  |
| 4x4 | anonymous entry | full+reply | yes | 3279 | 18 | 2 | - | 2489 | 2543 | yes |  |
| 4x4 | anonymous entry | full+2tree | yes | 3024 | 19 | 2 | - | 2201 | 2256 | yes |  |
| 4x4 | anonymous entry | full+data+ring | yes | 3259 | 18 | 2 | - | 2469 | 2517 | yes |  |
| 4x4 | delegate | min | yes | 1808 | 12 | 1 | - | 1293 | 1382 | yes |  |
| 4x4 | delegate | full | yes | 2255 | 18 | 2 | - | 1468 | 1531 | yes |  |
| 4x4 | delegate | full+spl | yes | 2495 | 25 | 2 | - | 1472 | 1535 | yes |  |
| 4x4 | delegate | full+data | yes | 2432 | 18 | 2 | - | 1645 | 1700 | yes |  |
| 4x4 | delegate | full+reply | yes | 2255 | 18 | 2 | - | 1468 | 1531 | yes |  |
| 4x4 | delegate | full+2tree | yes | 2293 | 19 | 2 | - | 1473 | 1535 | yes |  |
| 4x4 | delegate | full+data+ring | yes | 2528 | 18 | 2 | - | 1741 | 1796 | yes |  |
| 4x4 | anonymous delegate | min | yes | 1872 | 12 | 1 | - | 1357 | 1454 | yes |  |
| 4x4 | anonymous delegate | full | yes | 2319 | 18 | 2 | - | 1532 | 1603 | yes |  |
| 4x4 | anonymous delegate | full+spl | yes | 2559 | 25 | 2 | - | 1536 | 1607 | yes |  |
| 4x4 | anonymous delegate | full+data | yes | 2496 | 18 | 2 | - | 1709 | 1772 | yes |  |
| 4x4 | anonymous delegate | full+reply | yes | 2612 | 18 | 2 | - | 1825 | 1894 | yes |  |
| 4x4 | anonymous delegate | full+2tree | yes | 2357 | 19 | 2 | - | 1537 | 1607 | yes |  |
| 4x4 | anonymous delegate | full+data+ring | yes | 2592 | 18 | 2 | - | 1805 | 1868 | yes |  |
| 4x8 | confidential signer | min | yes | 2516 | 12 | 1 | - | 2001 | 2234 | yes |  |
| 4x8 | confidential signer | full | yes | 3175 | 22 | 3 | 4 | 2192 | 2383 | yes |  |
| 4x8 | confidential signer | full+spl | yes | 3543 | 29 | 3 | - | 2324 | 2387 | yes |  |
| 4x8 | confidential signer | full+data | yes | 3496 | 22 | 3 | - | 2513 | 2688 | yes |  |
| 4x8 | confidential signer | full+reply | yes | 3175 | 22 | 3 | - | 2192 | 2383 | yes |  |
| 4x8 | confidential signer | full+2tree | yes | 3213 | 23 | 3 | - | 2197 | 2387 | yes |  |
| 4x8 | confidential signer | full+data+ring | yes | 3720 | 22 | 3 | - | 2737 | 2912 | yes |  |
| 4x8 | confidential signer | full+data+program | yes | 3463 | 19 | 2 | - | 2477 | 2652 | yes |  |
| 4x8 | confidential signer | full+data+ring+program | yes | 3687 | 19 | 2 | - | 2701 | 2876 | yes |  |
| 4x8 | confidential in-proof | min | yes | 2611 | 12 | 1 | - | 2097 | 2331 | yes |  |
| 4x8 | confidential in-proof | full | yes | 3174 | 21 | 2 | 4 | 2288 | 2480 | yes |  |
| 4x8 | confidential in-proof | full+spl | yes | 3542 | 28 | 2 | - | 2420 | 2484 | yes |  |
| 4x8 | confidential in-proof | full+data | yes | 3495 | 21 | 2 | - | 2609 | 2785 | yes |  |
| 4x8 | confidential in-proof | full+reply | yes | 3174 | 21 | 2 | - | 2288 | 2480 | yes |  |
| 4x8 | confidential in-proof | full+2tree | yes | 3212 | 22 | 2 | - | 2293 | 2484 | yes |  |
| 4x8 | confidential in-proof | full+data+ring | yes | 3719 | 21 | 2 | - | 2833 | 3009 | yes |  |
| 4x8 | anonymous hidden | min | yes | 2643 | 12 | 1 | - | 2129 | 2403 | yes |  |
| 4x8 | anonymous hidden | full | yes | 3110 | 18 | 2 | 4 | 2320 | 2552 | yes |  |
| 4x8 | anonymous hidden | full+spl | yes | 3478 | 25 | 2 | - | 2452 | 2556 | yes |  |
| 4x8 | anonymous hidden | full+data | yes | 3431 | 18 | 2 | - | 2641 | 2857 | yes |  |
| 4x8 | anonymous hidden | full+reply | yes | 3791 | 18 | 2 | - | 3001 | 3231 | yes |  |
| 4x8 | anonymous hidden | full+2tree | yes | 3148 | 19 | 2 | - | 2325 | 2556 | yes |  |
| 4x8 | anonymous hidden | full+data+ring | yes | 3655 | 18 | 2 | - | 2865 | 3081 | yes |  |
| 4x8 | anonymous public owners | min | yes | 2580 | 12 | 1 | - | 2065 | 2306 | yes |  |
| 4x8 | anonymous public owners | full | yes | 3239 | 22 | 3 | 4 | 2256 | 2455 | yes |  |
| 4x8 | anonymous public owners | full+spl | yes | 3607 | 29 | 3 | - | 2388 | 2459 | yes |  |
| 4x8 | anonymous public owners | full+data | yes | 3560 | 22 | 3 | - | 2577 | 2760 | yes |  |
| 4x8 | anonymous public owners | full+reply | yes | 3920 | 22 | 3 | - | 2937 | 3134 | yes |  |
| 4x8 | anonymous public owners | full+2tree | yes | 3277 | 23 | 3 | - | 2261 | 2459 | yes |  |
| 4x8 | anonymous public owners | full+data+ring | yes | 3784 | 22 | 3 | - | 2801 | 2984 | yes |  |
| 4x8 | anonymous public owners | full+data+program | yes | 3527 | 19 | 2 | - | 2541 | 2724 | yes |  |
| 4x8 | anonymous public owners | full+data+ring+program | yes | 3751 | 19 | 2 | - | 2765 | 2948 | yes |  |
| 4x8 | confidential entry | min | yes | 3068 | 12 | 1 | - | 2553 | 2786 | yes |  |
| 4x8 | confidential entry | full | yes | 3727 | 22 | 3 | 4 | 2744 | 2935 | yes |  |
| 4x8 | confidential entry | full+spl | yes | 4095 | 29 | 3 | - | 2876 | 2939 | yes |  |
| 4x8 | confidential entry | full+data | yes | 4048 | 22 | 3 | - | 3065 | 3240 | yes |  |
| 4x8 | confidential entry | full+reply | yes | 3727 | 22 | 3 | - | 2744 | 2935 | yes |  |
| 4x8 | confidential entry | full+2tree | yes | 3765 | 23 | 3 | - | 2749 | 2939 | yes |  |
| 4x8 | confidential entry | full+data+ring | yes | 4272 | 22 | 3 | - | 3289 | 3464 | no | 176 B |
| 4x8 | anonymous entry | min | yes | 3227 | 12 | 1 | - | 2713 | 2955 | yes |  |
| 4x8 | anonymous entry | full | yes | 3694 | 18 | 2 | 4 | 2904 | 3104 | yes |  |
| 4x8 | anonymous entry | full+spl | yes | 4062 | 25 | 2 | - | 3036 | 3108 | yes |  |
| 4x8 | anonymous entry | full+data | yes | 4015 | 18 | 2 | - | 3225 | 3409 | yes |  |
| 4x8 | anonymous entry | full+reply | yes | 4375 | 18 | 2 | - | 3585 | 3783 | no | 279 B |
| 4x8 | anonymous entry | full+2tree | yes | 3732 | 19 | 2 | - | 2909 | 3108 | yes |  |
| 4x8 | anonymous entry | full+data+ring | yes | 4239 | 18 | 2 | - | 3449 | 3633 | no | 143 B |
| 4x8 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 4x8 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 5x2 | confidential signer | min | yes | 1519 | 13 | 1 | - | 971 | 989 | yes |  |
| 5x2 | confidential signer | full | yes | 2178 | 23 | 3 | 5 | 1162 | 1138 | yes |  |
| 5x2 | confidential signer | full+spl | yes | 2546 | 30 | 3 | - | 1294 | 1142 | yes |  |
| 5x2 | confidential signer | full+data | yes | 2283 | 23 | 3 | - | 1267 | 1239 | yes |  |
| 5x2 | confidential signer | full+reply | yes | 2178 | 23 | 3 | - | 1162 | 1138 | yes |  |
| 5x2 | confidential signer | full+2tree | yes | 2216 | 24 | 3 | - | 1167 | 1142 | yes |  |
| 5x2 | confidential signer | full+data+ring | yes | 2315 | 23 | 3 | - | 1299 | 1271 | yes |  |
| 5x2 | confidential signer | full+data+program | yes | 2250 | 20 | 2 | - | 1231 | 1203 | yes |  |
| 5x2 | confidential signer | full+data+ring+program | yes | 2282 | 20 | 2 | - | 1263 | 1235 | yes |  |
| 5x2 | confidential in-proof | min | yes | 1614 | 13 | 1 | - | 1067 | 1086 | yes |  |
| 5x2 | confidential in-proof | full | yes | 2177 | 22 | 2 | 5 | 1258 | 1235 | yes |  |
| 5x2 | confidential in-proof | full+spl | yes | 2545 | 29 | 2 | - | 1390 | 1239 | yes |  |
| 5x2 | confidential in-proof | full+data | yes | 2282 | 22 | 2 | - | 1363 | 1336 | yes |  |
| 5x2 | confidential in-proof | full+reply | yes | 2177 | 22 | 2 | - | 1258 | 1235 | yes |  |
| 5x2 | confidential in-proof | full+2tree | yes | 2215 | 23 | 2 | - | 1263 | 1239 | yes |  |
| 5x2 | confidential in-proof | full+data+ring | yes | 2314 | 22 | 2 | - | 1395 | 1368 | yes |  |
| 5x2 | anonymous hidden | min | yes | 1646 | 13 | 1 | - | 1099 | 1158 | yes |  |
| 5x2 | anonymous hidden | full | yes | 2113 | 19 | 2 | 5 | 1290 | 1307 | yes |  |
| 5x2 | anonymous hidden | full+spl | yes | 2481 | 26 | 2 | - | 1422 | 1311 | yes |  |
| 5x2 | anonymous hidden | full+data | yes | 2218 | 19 | 2 | - | 1395 | 1408 | yes |  |
| 5x2 | anonymous hidden | full+reply | yes | 2212 | 19 | 2 | - | 1389 | 1404 | yes |  |
| 5x2 | anonymous hidden | full+2tree | yes | 2151 | 20 | 2 | - | 1295 | 1311 | yes |  |
| 5x2 | anonymous hidden | full+data+ring | yes | 2250 | 19 | 2 | - | 1427 | 1440 | yes |  |
| 5x2 | anonymous public owners | min | yes | 1583 | 13 | 1 | - | 1035 | 1061 | yes |  |
| 5x2 | anonymous public owners | full | yes | 2242 | 23 | 3 | 5 | 1226 | 1210 | yes |  |
| 5x2 | anonymous public owners | full+spl | yes | 2610 | 30 | 3 | - | 1358 | 1214 | yes |  |
| 5x2 | anonymous public owners | full+data | yes | 2347 | 23 | 3 | - | 1331 | 1311 | yes |  |
| 5x2 | anonymous public owners | full+reply | yes | 2341 | 23 | 3 | - | 1325 | 1307 | yes |  |
| 5x2 | anonymous public owners | full+2tree | yes | 2280 | 24 | 3 | - | 1231 | 1214 | yes |  |
| 5x2 | anonymous public owners | full+data+ring | yes | 2379 | 23 | 3 | - | 1363 | 1343 | yes |  |
| 5x2 | anonymous public owners | full+data+program | yes | 2314 | 20 | 2 | - | 1295 | 1275 | yes |  |
| 5x2 | anonymous public owners | full+data+ring+program | yes | 2346 | 20 | 2 | - | 1327 | 1307 | yes |  |
| 5x2 | confidential entry | min | yes | 2207 | 13 | 1 | - | 1659 | 1677 | yes |  |
| 5x2 | confidential entry | full | yes | 2866 | 23 | 3 | 5 | 1850 | 1826 | yes |  |
| 5x2 | confidential entry | full+spl | yes | 3234 | 30 | 3 | - | 1982 | 1830 | yes |  |
| 5x2 | confidential entry | full+data | yes | 2971 | 23 | 3 | - | 1955 | 1927 | yes |  |
| 5x2 | confidential entry | full+reply | yes | 2866 | 23 | 3 | - | 1850 | 1826 | yes |  |
| 5x2 | confidential entry | full+2tree | yes | 2904 | 24 | 3 | - | 1855 | 1830 | yes |  |
| 5x2 | confidential entry | full+data+ring | yes | 3003 | 23 | 3 | - | 1987 | 1959 | yes |  |
| 5x2 | anonymous entry | min | yes | 2366 | 13 | 1 | - | 1819 | 1846 | yes |  |
| 5x2 | anonymous entry | full | yes | 2833 | 19 | 2 | 5 | 2010 | 1995 | yes |  |
| 5x2 | anonymous entry | full+spl | yes | 3201 | 26 | 2 | - | 2142 | 1999 | yes |  |
| 5x2 | anonymous entry | full+data | yes | 2938 | 19 | 2 | - | 2115 | 2096 | yes |  |
| 5x2 | anonymous entry | full+reply | yes | 2932 | 19 | 2 | - | 2109 | 2092 | yes |  |
| 5x2 | anonymous entry | full+2tree | yes | 2871 | 20 | 2 | - | 2015 | 1999 | yes |  |
| 5x2 | anonymous entry | full+data+ring | yes | 2970 | 19 | 2 | - | 2147 | 2128 | yes |  |
| 5x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 5x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 5x4 | confidential signer | min | yes | 1873 | 13 | 1 | - | 1325 | 1415 | yes |  |
| 5x4 | confidential signer | full | yes | 2532 | 23 | 3 | 5 | 1516 | 1564 | yes |  |
| 5x4 | confidential signer | full+spl | yes | 2900 | 30 | 3 | - | 1648 | 1568 | yes |  |
| 5x4 | confidential signer | full+data | yes | 2709 | 23 | 3 | - | 1693 | 1733 | yes |  |
| 5x4 | confidential signer | full+reply | yes | 2532 | 23 | 3 | - | 1516 | 1564 | yes |  |
| 5x4 | confidential signer | full+2tree | yes | 2570 | 24 | 3 | - | 1521 | 1568 | yes |  |
| 5x4 | confidential signer | full+data+ring | yes | 2805 | 23 | 3 | - | 1789 | 1829 | yes |  |
| 5x4 | confidential signer | full+data+program | yes | 2676 | 20 | 2 | - | 1657 | 1697 | yes |  |
| 5x4 | confidential signer | full+data+ring+program | yes | 2772 | 20 | 2 | - | 1753 | 1793 | yes |  |
| 5x4 | confidential in-proof | min | yes | 1968 | 13 | 1 | - | 1421 | 1512 | yes |  |
| 5x4 | confidential in-proof | full | yes | 2531 | 22 | 2 | 5 | 1612 | 1661 | yes |  |
| 5x4 | confidential in-proof | full+spl | yes | 2899 | 29 | 2 | - | 1744 | 1665 | yes |  |
| 5x4 | confidential in-proof | full+data | yes | 2708 | 22 | 2 | - | 1789 | 1830 | yes |  |
| 5x4 | confidential in-proof | full+reply | yes | 2531 | 22 | 2 | - | 1612 | 1661 | yes |  |
| 5x4 | confidential in-proof | full+2tree | yes | 2569 | 23 | 2 | - | 1617 | 1665 | yes |  |
| 5x4 | confidential in-proof | full+data+ring | yes | 2804 | 22 | 2 | - | 1885 | 1926 | yes |  |
| 5x4 | anonymous hidden | min | yes | 2000 | 13 | 1 | - | 1453 | 1584 | yes |  |
| 5x4 | anonymous hidden | full | yes | 2467 | 19 | 2 | 5 | 1644 | 1733 | yes |  |
| 5x4 | anonymous hidden | full+spl | yes | 2835 | 26 | 2 | - | 1776 | 1737 | yes |  |
| 5x4 | anonymous hidden | full+data | yes | 2644 | 19 | 2 | - | 1821 | 1902 | yes |  |
| 5x4 | anonymous hidden | full+reply | yes | 2760 | 19 | 2 | - | 1937 | 2024 | yes |  |
| 5x4 | anonymous hidden | full+2tree | yes | 2505 | 20 | 2 | - | 1649 | 1737 | yes |  |
| 5x4 | anonymous hidden | full+data+ring | yes | 2740 | 19 | 2 | - | 1917 | 1998 | yes |  |
| 5x4 | anonymous public owners | min | yes | 1937 | 13 | 1 | - | 1389 | 1487 | yes |  |
| 5x4 | anonymous public owners | full | yes | 2596 | 23 | 3 | 5 | 1580 | 1636 | yes |  |
| 5x4 | anonymous public owners | full+spl | yes | 2964 | 30 | 3 | - | 1712 | 1640 | yes |  |
| 5x4 | anonymous public owners | full+data | yes | 2773 | 23 | 3 | - | 1757 | 1805 | yes |  |
| 5x4 | anonymous public owners | full+reply | yes | 2889 | 23 | 3 | - | 1873 | 1927 | yes |  |
| 5x4 | anonymous public owners | full+2tree | yes | 2634 | 24 | 3 | - | 1585 | 1640 | yes |  |
| 5x4 | anonymous public owners | full+data+ring | yes | 2869 | 23 | 3 | - | 1853 | 1901 | yes |  |
| 5x4 | anonymous public owners | full+data+program | yes | 2740 | 20 | 2 | - | 1721 | 1769 | yes |  |
| 5x4 | anonymous public owners | full+data+ring+program | yes | 2836 | 20 | 2 | - | 1817 | 1865 | yes |  |
| 5x4 | confidential entry | min | yes | 2561 | 13 | 1 | - | 2013 | 2103 | yes |  |
| 5x4 | confidential entry | full | yes | 3220 | 23 | 3 | 5 | 2204 | 2252 | yes |  |
| 5x4 | confidential entry | full+spl | yes | 3588 | 30 | 3 | - | 2336 | 2256 | yes |  |
| 5x4 | confidential entry | full+data | yes | 3397 | 23 | 3 | - | 2381 | 2421 | yes |  |
| 5x4 | confidential entry | full+reply | yes | 3220 | 23 | 3 | - | 2204 | 2252 | yes |  |
| 5x4 | confidential entry | full+2tree | yes | 3258 | 24 | 3 | - | 2209 | 2256 | yes |  |
| 5x4 | confidential entry | full+data+ring | yes | 3493 | 23 | 3 | - | 2477 | 2517 | yes |  |
| 5x4 | anonymous entry | min | yes | 2720 | 13 | 1 | - | 2173 | 2272 | yes |  |
| 5x4 | anonymous entry | full | yes | 3187 | 19 | 2 | 5 | 2364 | 2421 | yes |  |
| 5x4 | anonymous entry | full+spl | yes | 3555 | 26 | 2 | - | 2496 | 2425 | yes |  |
| 5x4 | anonymous entry | full+data | yes | 3364 | 19 | 2 | - | 2541 | 2590 | yes |  |
| 5x4 | anonymous entry | full+reply | yes | 3480 | 19 | 2 | - | 2657 | 2712 | yes |  |
| 5x4 | anonymous entry | full+2tree | yes | 3225 | 20 | 2 | - | 2369 | 2425 | yes |  |
| 5x4 | anonymous entry | full+data+ring | yes | 3460 | 19 | 2 | - | 2637 | 2686 | yes |  |
| 5x4 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 5x4 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 4x16 | confidential signer | min | yes | 3933 | 12 | 1 | - | 3418 | 3938 | yes |  |
| 4x16 | confidential signer | full | yes | 4592 | 22 | 3 | none | 3609 | 4087 | no | 496 B |
| 4x16 | confidential signer | full+spl | yes | 4960 | 29 | 3 | - | 3741 | 4091 | no | 864 B |
| 4x16 | confidential signer | full+data | yes | 5201 | 22 | 3 | - | 4218 | 4664 | no | 1105 B |
| 4x16 | confidential signer | full+reply | yes | 4592 | 22 | 3 | - | 3609 | 4087 | no | 496 B |
| 4x16 | confidential signer | full+2tree | yes | 4630 | 23 | 3 | - | 3614 | 4091 | no | 534 B |
| 4x16 | confidential signer | full+data+ring | yes | 5681 | 22 | 3 | - | 4698 | 5144 | no | 1585 B |
| 4x16 | confidential signer | full+data+program | yes | 5168 | 19 | 2 | - | 4182 | 4628 | no | 1072 B |
| 4x16 | confidential signer | full+data+ring+program | yes | 5648 | 19 | 2 | - | 4662 | 5108 | no | 1552 B |
| 4x16 | confidential in-proof | min | yes | 4028 | 12 | 1 | - | 3514 | 4035 | yes |  |
| 4x16 | confidential in-proof | full | yes | 4591 | 21 | 2 | none | 3705 | 4184 | no | 495 B |
| 4x16 | confidential in-proof | full+spl | yes | 4959 | 28 | 2 | - | 3837 | 4188 | no | 863 B |
| 4x16 | confidential in-proof | full+data | yes | 5200 | 21 | 2 | - | 4314 | 4761 | no | 1104 B |
| 4x16 | confidential in-proof | full+reply | yes | 4591 | 21 | 2 | - | 3705 | 4184 | no | 495 B |
| 4x16 | confidential in-proof | full+2tree | yes | 4629 | 22 | 2 | - | 3710 | 4188 | no | 533 B |
| 4x16 | confidential in-proof | full+data+ring | yes | 5680 | 21 | 2 | - | 4794 | 5241 | no | 1584 B |
| 4x16 | anonymous hidden | min | yes | 4060 | 12 | 1 | - | 3546 | 4107 | yes |  |
| 4x16 | anonymous hidden | full | yes | 4527 | 18 | 2 | none | 3737 | 4256 | no | 431 B |
| 4x16 | anonymous hidden | full+spl | yes | 4895 | 25 | 2 | - | 3869 | 4260 | no | 799 B |
| 4x16 | anonymous hidden | full+data | yes | 5136 | 18 | 2 | - | 4346 | 4833 | no | 1040 B |
| 4x16 | anonymous hidden | full+reply | yes | 5984 | 18 | 2 | - | 5194 | 5711 | no | 1888 B |
| 4x16 | anonymous hidden | full+2tree | yes | 4565 | 19 | 2 | - | 3742 | 4260 | no | 469 B |
| 4x16 | anonymous hidden | full+data+ring | yes | 5616 | 18 | 2 | - | 4826 | 5313 | no | 1520 B |
| 4x16 | anonymous public owners | min | yes | 3997 | 12 | 1 | - | 3482 | 4010 | yes |  |
| 4x16 | anonymous public owners | full | yes | 4656 | 22 | 3 | none | 3673 | 4159 | no | 560 B |
| 4x16 | anonymous public owners | full+spl | yes | 5024 | 29 | 3 | - | 3805 | 4163 | no | 928 B |
| 4x16 | anonymous public owners | full+data | yes | 5265 | 22 | 3 | - | 4282 | 4736 | no | 1169 B |
| 4x16 | anonymous public owners | full+reply | yes | 6113 | 22 | 3 | - | 5130 | 5614 | no | 2017 B |
| 4x16 | anonymous public owners | full+2tree | yes | 4694 | 23 | 3 | - | 3678 | 4163 | no | 598 B |
| 4x16 | anonymous public owners | full+data+ring | yes | 5745 | 22 | 3 | - | 4762 | 5216 | no | 1649 B |
| 4x16 | anonymous public owners | full+data+program | yes | 5232 | 19 | 2 | - | 4246 | 4700 | no | 1136 B |
| 4x16 | anonymous public owners | full+data+ring+program | yes | 5712 | 19 | 2 | - | 4726 | 5180 | no | 1616 B |
| 4x16 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 4x16 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 4x16 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 5x8 | confidential signer | min | yes | 2581 | 13 | 1 | - | 2033 | 2267 | yes |  |
| 5x8 | confidential signer | full | yes | 3240 | 23 | 3 | 5 | 2224 | 2416 | yes |  |
| 5x8 | confidential signer | full+spl | yes | 3608 | 30 | 3 | - | 2356 | 2420 | yes |  |
| 5x8 | confidential signer | full+data | yes | 3561 | 23 | 3 | - | 2545 | 2721 | yes |  |
| 5x8 | confidential signer | full+reply | yes | 3240 | 23 | 3 | - | 2224 | 2416 | yes |  |
| 5x8 | confidential signer | full+2tree | yes | 3278 | 24 | 3 | - | 2229 | 2420 | yes |  |
| 5x8 | confidential signer | full+data+ring | yes | 3785 | 23 | 3 | - | 2769 | 2945 | yes |  |
| 5x8 | confidential signer | full+data+program | yes | 3528 | 20 | 2 | - | 2509 | 2685 | yes |  |
| 5x8 | confidential signer | full+data+ring+program | yes | 3752 | 20 | 2 | - | 2733 | 2909 | yes |  |
| 5x8 | confidential in-proof | min | yes | 2676 | 13 | 1 | - | 2129 | 2364 | yes |  |
| 5x8 | confidential in-proof | full | yes | 3239 | 22 | 2 | 5 | 2320 | 2513 | yes |  |
| 5x8 | confidential in-proof | full+spl | yes | 3607 | 29 | 2 | - | 2452 | 2517 | yes |  |
| 5x8 | confidential in-proof | full+data | yes | 3560 | 22 | 2 | - | 2641 | 2818 | yes |  |
| 5x8 | confidential in-proof | full+reply | yes | 3239 | 22 | 2 | - | 2320 | 2513 | yes |  |
| 5x8 | confidential in-proof | full+2tree | yes | 3277 | 23 | 2 | - | 2325 | 2517 | yes |  |
| 5x8 | confidential in-proof | full+data+ring | yes | 3784 | 22 | 2 | - | 2865 | 3042 | yes |  |
| 5x8 | anonymous hidden | min | yes | 2708 | 13 | 1 | - | 2161 | 2436 | yes |  |
| 5x8 | anonymous hidden | full | yes | 3175 | 19 | 2 | 5 | 2352 | 2585 | yes |  |
| 5x8 | anonymous hidden | full+spl | yes | 3543 | 26 | 2 | - | 2484 | 2589 | yes |  |
| 5x8 | anonymous hidden | full+data | yes | 3496 | 19 | 2 | - | 2673 | 2890 | yes |  |
| 5x8 | anonymous hidden | full+reply | yes | 3856 | 19 | 2 | - | 3033 | 3264 | yes |  |
| 5x8 | anonymous hidden | full+2tree | yes | 3213 | 20 | 2 | - | 2357 | 2589 | yes |  |
| 5x8 | anonymous hidden | full+data+ring | yes | 3720 | 19 | 2 | - | 2897 | 3114 | yes |  |
| 5x8 | anonymous public owners | min | yes | 2645 | 13 | 1 | - | 2097 | 2339 | yes |  |
| 5x8 | anonymous public owners | full | yes | 3304 | 23 | 3 | 5 | 2288 | 2488 | yes |  |
| 5x8 | anonymous public owners | full+spl | yes | 3672 | 30 | 3 | - | 2420 | 2492 | yes |  |
| 5x8 | anonymous public owners | full+data | yes | 3625 | 23 | 3 | - | 2609 | 2793 | yes |  |
| 5x8 | anonymous public owners | full+reply | yes | 3985 | 23 | 3 | - | 2969 | 3167 | yes |  |
| 5x8 | anonymous public owners | full+2tree | yes | 3342 | 24 | 3 | - | 2293 | 2492 | yes |  |
| 5x8 | anonymous public owners | full+data+ring | yes | 3849 | 23 | 3 | - | 2833 | 3017 | yes |  |
| 5x8 | anonymous public owners | full+data+program | yes | 3592 | 20 | 2 | - | 2573 | 2757 | yes |  |
| 5x8 | anonymous public owners | full+data+ring+program | yes | 3816 | 20 | 2 | - | 2797 | 2981 | yes |  |
| 5x8 | confidential entry | min | yes | 3269 | 13 | 1 | - | 2721 | 2955 | yes |  |
| 5x8 | confidential entry | full | yes | 3928 | 23 | 3 | 2 | 2912 | 3104 | yes |  |
| 5x8 | confidential entry | full+spl | yes | 4296 | 30 | 3 | - | 3044 | 3108 | no | 200 B |
| 5x8 | confidential entry | full+data | yes | 4249 | 23 | 3 | - | 3233 | 3409 | no | 153 B |
| 5x8 | confidential entry | full+reply | yes | 3928 | 23 | 3 | - | 2912 | 3104 | yes |  |
| 5x8 | confidential entry | full+2tree | yes | 3966 | 24 | 3 | - | 2917 | 3108 | yes |  |
| 5x8 | confidential entry | full+data+ring | yes | 4473 | 23 | 3 | - | 3457 | 3633 | no | 377 B |
| 5x8 | anonymous entry | min | yes | 3428 | 13 | 1 | - | 2881 | 3124 | yes |  |
| 5x8 | anonymous entry | full | yes | 3895 | 19 | 2 | 2 | 3072 | 3273 | yes |  |
| 5x8 | anonymous entry | full+spl | yes | 4263 | 26 | 2 | - | 3204 | 3277 | no | 167 B |
| 5x8 | anonymous entry | full+data | yes | 4216 | 19 | 2 | - | 3393 | 3578 | no | 120 B |
| 5x8 | anonymous entry | full+reply | yes | 4576 | 19 | 2 | - | 3753 | 3952 | no | 480 B |
| 5x8 | anonymous entry | full+2tree | yes | 3933 | 20 | 2 | - | 3077 | 3277 | yes |  |
| 5x8 | anonymous entry | full+data+ring | yes | 4440 | 19 | 2 | - | 3617 | 3802 | no | 344 B |
| 5x8 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 5x8 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 6x2 | confidential signer | min | yes | 1584 | 14 | 1 | - | 1003 | 1022 | yes |  |
| 6x2 | confidential signer | full | yes | 2243 | 24 | 3 | 6 | 1194 | 1171 | yes |  |
| 6x2 | confidential signer | full+spl | yes | 2611 | 31 | 3 | - | 1326 | 1175 | yes |  |
| 6x2 | confidential signer | full+data | yes | 2348 | 24 | 3 | - | 1299 | 1272 | yes |  |
| 6x2 | confidential signer | full+reply | yes | 2243 | 24 | 3 | - | 1194 | 1171 | yes |  |
| 6x2 | confidential signer | full+2tree | yes | 2281 | 25 | 3 | - | 1199 | 1175 | yes |  |
| 6x2 | confidential signer | full+data+ring | yes | 2380 | 24 | 3 | - | 1331 | 1304 | yes |  |
| 6x2 | confidential signer | full+data+program | yes | 2315 | 21 | 2 | - | 1263 | 1236 | yes |  |
| 6x2 | confidential signer | full+data+ring+program | yes | 2347 | 21 | 2 | - | 1295 | 1268 | yes |  |
| 6x2 | confidential in-proof | min | yes | 1679 | 14 | 1 | - | 1099 | 1119 | yes |  |
| 6x2 | confidential in-proof | full | yes | 2242 | 23 | 2 | 6 | 1290 | 1268 | yes |  |
| 6x2 | confidential in-proof | full+spl | yes | 2610 | 30 | 2 | - | 1422 | 1272 | yes |  |
| 6x2 | confidential in-proof | full+data | yes | 2347 | 23 | 2 | - | 1395 | 1369 | yes |  |
| 6x2 | confidential in-proof | full+reply | yes | 2242 | 23 | 2 | - | 1290 | 1268 | yes |  |
| 6x2 | confidential in-proof | full+2tree | yes | 2280 | 24 | 2 | - | 1295 | 1272 | yes |  |
| 6x2 | confidential in-proof | full+data+ring | yes | 2379 | 23 | 2 | - | 1427 | 1401 | yes |  |
| 6x2 | anonymous hidden | min | yes | 1711 | 14 | 1 | - | 1131 | 1191 | yes |  |
| 6x2 | anonymous hidden | full | yes | 2178 | 20 | 2 | 6 | 1322 | 1340 | yes |  |
| 6x2 | anonymous hidden | full+spl | yes | 2546 | 27 | 2 | - | 1454 | 1344 | yes |  |
| 6x2 | anonymous hidden | full+data | yes | 2283 | 20 | 2 | - | 1427 | 1441 | yes |  |
| 6x2 | anonymous hidden | full+reply | yes | 2277 | 20 | 2 | - | 1421 | 1437 | yes |  |
| 6x2 | anonymous hidden | full+2tree | yes | 2216 | 21 | 2 | - | 1327 | 1344 | yes |  |
| 6x2 | anonymous hidden | full+data+ring | yes | 2315 | 20 | 2 | - | 1459 | 1473 | yes |  |
| 6x2 | anonymous public owners | min | yes | 1648 | 14 | 1 | - | 1067 | 1094 | yes |  |
| 6x2 | anonymous public owners | full | yes | 2307 | 24 | 3 | 6 | 1258 | 1243 | yes |  |
| 6x2 | anonymous public owners | full+spl | yes | 2675 | 31 | 3 | - | 1390 | 1247 | yes |  |
| 6x2 | anonymous public owners | full+data | yes | 2412 | 24 | 3 | - | 1363 | 1344 | yes |  |
| 6x2 | anonymous public owners | full+reply | yes | 2406 | 24 | 3 | - | 1357 | 1340 | yes |  |
| 6x2 | anonymous public owners | full+2tree | yes | 2345 | 25 | 3 | - | 1263 | 1247 | yes |  |
| 6x2 | anonymous public owners | full+data+ring | yes | 2444 | 24 | 3 | - | 1395 | 1376 | yes |  |
| 6x2 | anonymous public owners | full+data+program | yes | 2379 | 21 | 2 | - | 1327 | 1308 | yes |  |
| 6x2 | anonymous public owners | full+data+ring+program | yes | 2411 | 21 | 2 | - | 1359 | 1340 | yes |  |
| 6x2 | confidential entry | min | yes | 2408 | 14 | 1 | - | 1827 | 1846 | yes |  |
| 6x2 | confidential entry | full | yes | 3067 | 24 | 3 | 6 | 2018 | 1995 | yes |  |
| 6x2 | confidential entry | full+spl | yes | 3435 | 31 | 3 | - | 2150 | 1999 | yes |  |
| 6x2 | confidential entry | full+data | yes | 3172 | 24 | 3 | - | 2123 | 2096 | yes |  |
| 6x2 | confidential entry | full+reply | yes | 3067 | 24 | 3 | - | 2018 | 1995 | yes |  |
| 6x2 | confidential entry | full+2tree | yes | 3105 | 25 | 3 | - | 2023 | 1999 | yes |  |
| 6x2 | confidential entry | full+data+ring | yes | 3204 | 24 | 3 | - | 2155 | 2128 | yes |  |
| 6x2 | anonymous entry | min | yes | 2567 | 14 | 1 | - | 1987 | 2015 | yes |  |
| 6x2 | anonymous entry | full | yes | 3034 | 20 | 2 | 6 | 2178 | 2164 | yes |  |
| 6x2 | anonymous entry | full+spl | yes | 3402 | 27 | 2 | - | 2310 | 2168 | yes |  |
| 6x2 | anonymous entry | full+data | yes | 3139 | 20 | 2 | - | 2283 | 2265 | yes |  |
| 6x2 | anonymous entry | full+reply | yes | 3133 | 20 | 2 | - | 2277 | 2261 | yes |  |
| 6x2 | anonymous entry | full+2tree | yes | 3072 | 21 | 2 | - | 2183 | 2168 | yes |  |
| 6x2 | anonymous entry | full+data+ring | yes | 3171 | 20 | 2 | - | 2315 | 2297 | yes |  |
| 6x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 6x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 6x4 | confidential signer | min | yes | 1938 | 14 | 1 | - | 1357 | 1448 | yes |  |
| 6x4 | confidential signer | full | yes | 2597 | 24 | 3 | 6 | 1548 | 1597 | yes |  |
| 6x4 | confidential signer | full+spl | yes | 2965 | 31 | 3 | - | 1680 | 1601 | yes |  |
| 6x4 | confidential signer | full+data | yes | 2774 | 24 | 3 | - | 1725 | 1766 | yes |  |
| 6x4 | confidential signer | full+reply | yes | 2597 | 24 | 3 | - | 1548 | 1597 | yes |  |
| 6x4 | confidential signer | full+2tree | yes | 2635 | 25 | 3 | - | 1553 | 1601 | yes |  |
| 6x4 | confidential signer | full+data+ring | yes | 2870 | 24 | 3 | - | 1821 | 1862 | yes |  |
| 6x4 | confidential signer | full+data+program | yes | 2741 | 21 | 2 | - | 1689 | 1730 | yes |  |
| 6x4 | confidential signer | full+data+ring+program | yes | 2837 | 21 | 2 | - | 1785 | 1826 | yes |  |
| 6x4 | confidential in-proof | min | yes | 2033 | 14 | 1 | - | 1453 | 1545 | yes |  |
| 6x4 | confidential in-proof | full | yes | 2596 | 23 | 2 | 6 | 1644 | 1694 | yes |  |
| 6x4 | confidential in-proof | full+spl | yes | 2964 | 30 | 2 | - | 1776 | 1698 | yes |  |
| 6x4 | confidential in-proof | full+data | yes | 2773 | 23 | 2 | - | 1821 | 1863 | yes |  |
| 6x4 | confidential in-proof | full+reply | yes | 2596 | 23 | 2 | - | 1644 | 1694 | yes |  |
| 6x4 | confidential in-proof | full+2tree | yes | 2634 | 24 | 2 | - | 1649 | 1698 | yes |  |
| 6x4 | confidential in-proof | full+data+ring | yes | 2869 | 23 | 2 | - | 1917 | 1959 | yes |  |
| 6x4 | anonymous hidden | min | yes | 2065 | 14 | 1 | - | 1485 | 1617 | yes |  |
| 6x4 | anonymous hidden | full | yes | 2532 | 20 | 2 | 6 | 1676 | 1766 | yes |  |
| 6x4 | anonymous hidden | full+spl | yes | 2900 | 27 | 2 | - | 1808 | 1770 | yes |  |
| 6x4 | anonymous hidden | full+data | yes | 2709 | 20 | 2 | - | 1853 | 1935 | yes |  |
| 6x4 | anonymous hidden | full+reply | yes | 2825 | 20 | 2 | - | 1969 | 2057 | yes |  |
| 6x4 | anonymous hidden | full+2tree | yes | 2570 | 21 | 2 | - | 1681 | 1770 | yes |  |
| 6x4 | anonymous hidden | full+data+ring | yes | 2805 | 20 | 2 | - | 1949 | 2031 | yes |  |
| 6x4 | anonymous public owners | min | yes | 2002 | 14 | 1 | - | 1421 | 1520 | yes |  |
| 6x4 | anonymous public owners | full | yes | 2661 | 24 | 3 | 6 | 1612 | 1669 | yes |  |
| 6x4 | anonymous public owners | full+spl | yes | 3029 | 31 | 3 | - | 1744 | 1673 | yes |  |
| 6x4 | anonymous public owners | full+data | yes | 2838 | 24 | 3 | - | 1789 | 1838 | yes |  |
| 6x4 | anonymous public owners | full+reply | yes | 2954 | 24 | 3 | - | 1905 | 1960 | yes |  |
| 6x4 | anonymous public owners | full+2tree | yes | 2699 | 25 | 3 | - | 1617 | 1673 | yes |  |
| 6x4 | anonymous public owners | full+data+ring | yes | 2934 | 24 | 3 | - | 1885 | 1934 | yes |  |
| 6x4 | anonymous public owners | full+data+program | yes | 2805 | 21 | 2 | - | 1753 | 1802 | yes |  |
| 6x4 | anonymous public owners | full+data+ring+program | yes | 2901 | 21 | 2 | - | 1849 | 1898 | yes |  |
| 6x4 | confidential entry | min | yes | 2762 | 14 | 1 | - | 2181 | 2272 | yes |  |
| 6x4 | confidential entry | full | yes | 3421 | 24 | 3 | 6 | 2372 | 2421 | yes |  |
| 6x4 | confidential entry | full+spl | yes | 3789 | 31 | 3 | - | 2504 | 2425 | yes |  |
| 6x4 | confidential entry | full+data | yes | 3598 | 24 | 3 | - | 2549 | 2590 | yes |  |
| 6x4 | confidential entry | full+reply | yes | 3421 | 24 | 3 | - | 2372 | 2421 | yes |  |
| 6x4 | confidential entry | full+2tree | yes | 3459 | 25 | 3 | - | 2377 | 2425 | yes |  |
| 6x4 | confidential entry | full+data+ring | yes | 3694 | 24 | 3 | - | 2645 | 2686 | yes |  |
| 6x4 | anonymous entry | min | yes | 2921 | 14 | 1 | - | 2341 | 2441 | yes |  |
| 6x4 | anonymous entry | full | yes | 3388 | 20 | 2 | 6 | 2532 | 2590 | yes |  |
| 6x4 | anonymous entry | full+spl | yes | 3756 | 27 | 2 | - | 2664 | 2594 | yes |  |
| 6x4 | anonymous entry | full+data | yes | 3565 | 20 | 2 | - | 2709 | 2759 | yes |  |
| 6x4 | anonymous entry | full+reply | yes | 3681 | 20 | 2 | - | 2825 | 2881 | yes |  |
| 6x4 | anonymous entry | full+2tree | yes | 3426 | 21 | 2 | - | 2537 | 2594 | yes |  |
| 6x4 | anonymous entry | full+data+ring | yes | 3661 | 20 | 2 | - | 2805 | 2855 | yes |  |
| 6x4 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 6x4 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 5x16 | confidential signer | min | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential signer | full | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential signer | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential signer | full+data | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential signer | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential signer | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential signer | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential signer | full+data+program | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential signer | full+data+ring+program | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential in-proof | min | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential in-proof | full | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential in-proof | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential in-proof | full+data | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential in-proof | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential in-proof | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential in-proof | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous hidden | min | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous hidden | full | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous hidden | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous hidden | full+data | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous hidden | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous hidden | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous hidden | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous public owners | min | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous public owners | full | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous public owners | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous public owners | full+data | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous public owners | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous public owners | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous public owners | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous public owners | full+data+program | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous public owners | full+data+ring+program | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 5x16 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 5x16 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 6x8 | confidential signer | min | yes | 2646 | 14 | 1 | - | 2065 | 2300 | yes |  |
| 6x8 | confidential signer | full | yes | 3305 | 24 | 3 | 6 | 2256 | 2449 | yes |  |
| 6x8 | confidential signer | full+spl | yes | 3673 | 31 | 3 | - | 2388 | 2453 | yes |  |
| 6x8 | confidential signer | full+data | yes | 3626 | 24 | 3 | - | 2577 | 2754 | yes |  |
| 6x8 | confidential signer | full+reply | yes | 3305 | 24 | 3 | - | 2256 | 2449 | yes |  |
| 6x8 | confidential signer | full+2tree | yes | 3343 | 25 | 3 | - | 2261 | 2453 | yes |  |
| 6x8 | confidential signer | full+data+ring | yes | 3850 | 24 | 3 | - | 2801 | 2978 | yes |  |
| 6x8 | confidential signer | full+data+program | yes | 3593 | 21 | 2 | - | 2541 | 2718 | yes |  |
| 6x8 | confidential signer | full+data+ring+program | yes | 3817 | 21 | 2 | - | 2765 | 2942 | yes |  |
| 6x8 | confidential in-proof | min | yes | 2741 | 14 | 1 | - | 2161 | 2397 | yes |  |
| 6x8 | confidential in-proof | full | yes | 3304 | 23 | 2 | 6 | 2352 | 2546 | yes |  |
| 6x8 | confidential in-proof | full+spl | yes | 3672 | 30 | 2 | - | 2484 | 2550 | yes |  |
| 6x8 | confidential in-proof | full+data | yes | 3625 | 23 | 2 | - | 2673 | 2851 | yes |  |
| 6x8 | confidential in-proof | full+reply | yes | 3304 | 23 | 2 | - | 2352 | 2546 | yes |  |
| 6x8 | confidential in-proof | full+2tree | yes | 3342 | 24 | 2 | - | 2357 | 2550 | yes |  |
| 6x8 | confidential in-proof | full+data+ring | yes | 3849 | 23 | 2 | - | 2897 | 3075 | yes |  |
| 6x8 | anonymous hidden | min | yes | 2773 | 14 | 1 | - | 2193 | 2469 | yes |  |
| 6x8 | anonymous hidden | full | yes | 3240 | 20 | 2 | 6 | 2384 | 2618 | yes |  |
| 6x8 | anonymous hidden | full+spl | yes | 3608 | 27 | 2 | - | 2516 | 2622 | yes |  |
| 6x8 | anonymous hidden | full+data | yes | 3561 | 20 | 2 | - | 2705 | 2923 | yes |  |
| 6x8 | anonymous hidden | full+reply | yes | 3921 | 20 | 2 | - | 3065 | 3297 | yes |  |
| 6x8 | anonymous hidden | full+2tree | yes | 3278 | 21 | 2 | - | 2389 | 2622 | yes |  |
| 6x8 | anonymous hidden | full+data+ring | yes | 3785 | 20 | 2 | - | 2929 | 3147 | yes |  |
| 6x8 | anonymous public owners | min | yes | 2710 | 14 | 1 | - | 2129 | 2372 | yes |  |
| 6x8 | anonymous public owners | full | yes | 3369 | 24 | 3 | 6 | 2320 | 2521 | yes |  |
| 6x8 | anonymous public owners | full+spl | yes | 3737 | 31 | 3 | - | 2452 | 2525 | yes |  |
| 6x8 | anonymous public owners | full+data | yes | 3690 | 24 | 3 | - | 2641 | 2826 | yes |  |
| 6x8 | anonymous public owners | full+reply | yes | 4050 | 24 | 3 | - | 3001 | 3200 | yes |  |
| 6x8 | anonymous public owners | full+2tree | yes | 3407 | 25 | 3 | - | 2325 | 2525 | yes |  |
| 6x8 | anonymous public owners | full+data+ring | yes | 3914 | 24 | 3 | - | 2865 | 3050 | yes |  |
| 6x8 | anonymous public owners | full+data+program | yes | 3657 | 21 | 2 | - | 2605 | 2790 | yes |  |
| 6x8 | anonymous public owners | full+data+ring+program | yes | 3881 | 21 | 2 | - | 2829 | 3014 | yes |  |
| 6x8 | confidential entry | min | yes | 3470 | 14 | 1 | - | 2889 | 3124 | yes |  |
| 6x8 | confidential entry | full | yes | 4129 | 24 | 3 | none | 3080 | 3273 | no | 33 B |
| 6x8 | confidential entry | full+spl | yes | 4497 | 31 | 3 | - | 3212 | 3277 | no | 401 B |
| 6x8 | confidential entry | full+data | yes | 4450 | 24 | 3 | - | 3401 | 3578 | no | 354 B |
| 6x8 | confidential entry | full+reply | yes | 4129 | 24 | 3 | - | 3080 | 3273 | no | 33 B |
| 6x8 | confidential entry | full+2tree | yes | 4167 | 25 | 3 | - | 3085 | 3277 | no | 71 B |
| 6x8 | confidential entry | full+data+ring | yes | 4674 | 24 | 3 | - | 3625 | 3802 | no | 578 B |
| 6x8 | anonymous entry | min | yes | 3629 | 14 | 1 | - | 3049 | 3293 | yes |  |
| 6x8 | anonymous entry | full | yes | 4096 | 20 | 2 | 0 | 3240 | 3442 | yes |  |
| 6x8 | anonymous entry | full+spl | yes | 4464 | 27 | 2 | - | 3372 | 3446 | no | 368 B |
| 6x8 | anonymous entry | full+data | yes | 4417 | 20 | 2 | - | 3561 | 3747 | no | 321 B |
| 6x8 | anonymous entry | full+reply | yes | 4777 | 20 | 2 | - | 3921 | 4121 | no | 681 B |
| 6x8 | anonymous entry | full+2tree | yes | 4134 | 21 | 2 | - | 3245 | 3446 | no | 38 B |
| 6x8 | anonymous entry | full+data+ring | yes | 4641 | 20 | 2 | - | 3785 | 3971 | no | 545 B |
| 6x8 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 6x8 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 8x2 | confidential signer | min | yes | 1714 | 16 | 1 | - | 1067 | 1088 | yes |  |
| 8x2 | confidential signer | full | yes | 2373 | 26 | 3 | 8 | 1258 | 1237 | yes |  |
| 8x2 | confidential signer | full+spl | yes | 2741 | 33 | 3 | - | 1390 | 1241 | yes |  |
| 8x2 | confidential signer | full+data | yes | 2478 | 26 | 3 | - | 1363 | 1338 | yes |  |
| 8x2 | confidential signer | full+reply | yes | 2373 | 26 | 3 | - | 1258 | 1237 | yes |  |
| 8x2 | confidential signer | full+2tree | yes | 2411 | 27 | 3 | - | 1263 | 1241 | yes |  |
| 8x2 | confidential signer | full+data+ring | yes | 2510 | 26 | 3 | - | 1395 | 1370 | yes |  |
| 8x2 | confidential signer | full+data+program | yes | 2445 | 23 | 2 | - | 1327 | 1302 | yes |  |
| 8x2 | confidential signer | full+data+ring+program | yes | 2477 | 23 | 2 | - | 1359 | 1334 | yes |  |
| 8x2 | confidential in-proof | min | yes | 1809 | 16 | 1 | - | 1163 | 1185 | yes |  |
| 8x2 | confidential in-proof | full | yes | 2372 | 25 | 2 | 8 | 1354 | 1334 | yes |  |
| 8x2 | confidential in-proof | full+spl | yes | 2740 | 32 | 2 | - | 1486 | 1338 | yes |  |
| 8x2 | confidential in-proof | full+data | yes | 2477 | 25 | 2 | - | 1459 | 1435 | yes |  |
| 8x2 | confidential in-proof | full+reply | yes | 2372 | 25 | 2 | - | 1354 | 1334 | yes |  |
| 8x2 | confidential in-proof | full+2tree | yes | 2410 | 26 | 2 | - | 1359 | 1338 | yes |  |
| 8x2 | confidential in-proof | full+data+ring | yes | 2509 | 25 | 2 | - | 1491 | 1467 | yes |  |
| 8x2 | anonymous hidden | min | yes | 1841 | 16 | 1 | - | 1195 | 1257 | yes |  |
| 8x2 | anonymous hidden | full | yes | 2308 | 22 | 2 | 8 | 1386 | 1406 | yes |  |
| 8x2 | anonymous hidden | full+spl | yes | 2676 | 29 | 2 | - | 1518 | 1410 | yes |  |
| 8x2 | anonymous hidden | full+data | yes | 2413 | 22 | 2 | - | 1491 | 1507 | yes |  |
| 8x2 | anonymous hidden | full+reply | yes | 2407 | 22 | 2 | - | 1485 | 1503 | yes |  |
| 8x2 | anonymous hidden | full+2tree | yes | 2346 | 23 | 2 | - | 1391 | 1410 | yes |  |
| 8x2 | anonymous hidden | full+data+ring | yes | 2445 | 22 | 2 | - | 1523 | 1539 | yes |  |
| 8x2 | anonymous public owners | min | yes | 1778 | 16 | 1 | - | 1131 | 1160 | yes |  |
| 8x2 | anonymous public owners | full | yes | 2437 | 26 | 3 | 8 | 1322 | 1309 | yes |  |
| 8x2 | anonymous public owners | full+spl | yes | 2805 | 33 | 3 | - | 1454 | 1313 | yes |  |
| 8x2 | anonymous public owners | full+data | yes | 2542 | 26 | 3 | - | 1427 | 1410 | yes |  |
| 8x2 | anonymous public owners | full+reply | yes | 2536 | 26 | 3 | - | 1421 | 1406 | yes |  |
| 8x2 | anonymous public owners | full+2tree | yes | 2475 | 27 | 3 | - | 1327 | 1313 | yes |  |
| 8x2 | anonymous public owners | full+data+ring | yes | 2574 | 26 | 3 | - | 1459 | 1442 | yes |  |
| 8x2 | anonymous public owners | full+data+program | yes | 2509 | 23 | 2 | - | 1391 | 1374 | yes |  |
| 8x2 | anonymous public owners | full+data+ring+program | yes | 2541 | 23 | 2 | - | 1423 | 1406 | yes |  |
| 8x2 | confidential entry | min | yes | 2810 | 16 | 1 | - | 2163 | 2184 | yes |  |
| 8x2 | confidential entry | full | yes | 3469 | 26 | 3 | 7 | 2354 | 2333 | yes |  |
| 8x2 | confidential entry | full+spl | yes | 3837 | 33 | 3 | - | 2486 | 2337 | yes |  |
| 8x2 | confidential entry | full+data | yes | 3574 | 26 | 3 | - | 2459 | 2434 | yes |  |
| 8x2 | confidential entry | full+reply | yes | 3469 | 26 | 3 | - | 2354 | 2333 | yes |  |
| 8x2 | confidential entry | full+2tree | yes | 3507 | 27 | 3 | - | 2359 | 2337 | yes |  |
| 8x2 | confidential entry | full+data+ring | yes | 3606 | 26 | 3 | - | 2491 | 2466 | yes |  |
| 8x2 | anonymous entry | min | yes | 2969 | 16 | 1 | - | 2323 | 2353 | yes |  |
| 8x2 | anonymous entry | full | yes | 3436 | 22 | 2 | 6 | 2514 | 2502 | yes |  |
| 8x2 | anonymous entry | full+spl | yes | 3804 | 29 | 2 | - | 2646 | 2506 | yes |  |
| 8x2 | anonymous entry | full+data | yes | 3541 | 22 | 2 | - | 2619 | 2603 | yes |  |
| 8x2 | anonymous entry | full+reply | yes | 3535 | 22 | 2 | - | 2613 | 2599 | yes |  |
| 8x2 | anonymous entry | full+2tree | yes | 3474 | 23 | 2 | - | 2519 | 2506 | yes |  |
| 8x2 | anonymous entry | full+data+ring | yes | 3573 | 22 | 2 | - | 2651 | 2635 | yes |  |
| 8x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 8x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 8x4 | confidential signer | min | yes | 2068 | 16 | 1 | - | 1421 | 1514 | yes |  |
| 8x4 | confidential signer | full | yes | 2727 | 26 | 3 | 8 | 1612 | 1663 | yes |  |
| 8x4 | confidential signer | full+spl | yes | 3095 | 33 | 3 | - | 1744 | 1667 | yes |  |
| 8x4 | confidential signer | full+data | yes | 2904 | 26 | 3 | - | 1789 | 1832 | yes |  |
| 8x4 | confidential signer | full+reply | yes | 2727 | 26 | 3 | - | 1612 | 1663 | yes |  |
| 8x4 | confidential signer | full+2tree | yes | 2765 | 27 | 3 | - | 1617 | 1667 | yes |  |
| 8x4 | confidential signer | full+data+ring | yes | 3000 | 26 | 3 | - | 1885 | 1928 | yes |  |
| 8x4 | confidential signer | full+data+program | yes | 2871 | 23 | 2 | - | 1753 | 1796 | yes |  |
| 8x4 | confidential signer | full+data+ring+program | yes | 2967 | 23 | 2 | - | 1849 | 1892 | yes |  |
| 8x4 | confidential in-proof | min | yes | 2163 | 16 | 1 | - | 1517 | 1611 | yes |  |
| 8x4 | confidential in-proof | full | yes | 2726 | 25 | 2 | 8 | 1708 | 1760 | yes |  |
| 8x4 | confidential in-proof | full+spl | yes | 3094 | 32 | 2 | - | 1840 | 1764 | yes |  |
| 8x4 | confidential in-proof | full+data | yes | 2903 | 25 | 2 | - | 1885 | 1929 | yes |  |
| 8x4 | confidential in-proof | full+reply | yes | 2726 | 25 | 2 | - | 1708 | 1760 | yes |  |
| 8x4 | confidential in-proof | full+2tree | yes | 2764 | 26 | 2 | - | 1713 | 1764 | yes |  |
| 8x4 | confidential in-proof | full+data+ring | yes | 2999 | 25 | 2 | - | 1981 | 2025 | yes |  |
| 8x4 | anonymous hidden | min | yes | 2195 | 16 | 1 | - | 1549 | 1683 | yes |  |
| 8x4 | anonymous hidden | full | yes | 2662 | 22 | 2 | 8 | 1740 | 1832 | yes |  |
| 8x4 | anonymous hidden | full+spl | yes | 3030 | 29 | 2 | - | 1872 | 1836 | yes |  |
| 8x4 | anonymous hidden | full+data | yes | 2839 | 22 | 2 | - | 1917 | 2001 | yes |  |
| 8x4 | anonymous hidden | full+reply | yes | 2955 | 22 | 2 | - | 2033 | 2123 | yes |  |
| 8x4 | anonymous hidden | full+2tree | yes | 2700 | 23 | 2 | - | 1745 | 1836 | yes |  |
| 8x4 | anonymous hidden | full+data+ring | yes | 2935 | 22 | 2 | - | 2013 | 2097 | yes |  |
| 8x4 | anonymous public owners | min | yes | 2132 | 16 | 1 | - | 1485 | 1586 | yes |  |
| 8x4 | anonymous public owners | full | yes | 2791 | 26 | 3 | 8 | 1676 | 1735 | yes |  |
| 8x4 | anonymous public owners | full+spl | yes | 3159 | 33 | 3 | - | 1808 | 1739 | yes |  |
| 8x4 | anonymous public owners | full+data | yes | 2968 | 26 | 3 | - | 1853 | 1904 | yes |  |
| 8x4 | anonymous public owners | full+reply | yes | 3084 | 26 | 3 | - | 1969 | 2026 | yes |  |
| 8x4 | anonymous public owners | full+2tree | yes | 2829 | 27 | 3 | - | 1681 | 1739 | yes |  |
| 8x4 | anonymous public owners | full+data+ring | yes | 3064 | 26 | 3 | - | 1949 | 2000 | yes |  |
| 8x4 | anonymous public owners | full+data+program | yes | 2935 | 23 | 2 | - | 1817 | 1868 | yes |  |
| 8x4 | anonymous public owners | full+data+ring+program | yes | 3031 | 23 | 2 | - | 1913 | 1964 | yes |  |
| 8x4 | confidential entry | min | yes | 3164 | 16 | 1 | - | 2517 | 2610 | yes |  |
| 8x4 | confidential entry | full | yes | 3823 | 26 | 3 | 3 | 2708 | 2759 | yes |  |
| 8x4 | confidential entry | full+spl | yes | 4191 | 33 | 3 | - | 2840 | 2763 | no | 95 B |
| 8x4 | confidential entry | full+data | yes | 4000 | 26 | 3 | - | 2885 | 2928 | yes |  |
| 8x4 | confidential entry | full+reply | yes | 3823 | 26 | 3 | - | 2708 | 2759 | yes |  |
| 8x4 | confidential entry | full+2tree | yes | 3861 | 27 | 3 | - | 2713 | 2763 | yes |  |
| 8x4 | confidential entry | full+data+ring | yes | 4096 | 26 | 3 | - | 2981 | 3024 | yes |  |
| 8x4 | anonymous entry | min | yes | 3323 | 16 | 1 | - | 2677 | 2779 | yes |  |
| 8x4 | anonymous entry | full | yes | 3790 | 22 | 2 | 3 | 2868 | 2928 | yes |  |
| 8x4 | anonymous entry | full+spl | yes | 4158 | 29 | 2 | - | 3000 | 2932 | no | 62 B |
| 8x4 | anonymous entry | full+data | yes | 3967 | 22 | 2 | - | 3045 | 3097 | yes |  |
| 8x4 | anonymous entry | full+reply | yes | 4083 | 22 | 2 | - | 3161 | 3219 | yes |  |
| 8x4 | anonymous entry | full+2tree | yes | 3828 | 23 | 2 | - | 2873 | 2932 | yes |  |
| 8x4 | anonymous entry | full+data+ring | yes | 4063 | 22 | 2 | - | 3141 | 3193 | yes |  |
| 8x4 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 8x4 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 8x8 | confidential signer | min | yes | 2776 | 16 | 1 | - | 2129 | 2366 | yes |  |
| 8x8 | confidential signer | full | yes | 3435 | 26 | 3 | 7 | 2320 | 2515 | yes |  |
| 8x8 | confidential signer | full+spl | yes | 3803 | 33 | 3 | - | 2452 | 2519 | yes |  |
| 8x8 | confidential signer | full+data | yes | 3756 | 26 | 3 | - | 2641 | 2820 | yes |  |
| 8x8 | confidential signer | full+reply | yes | 3435 | 26 | 3 | - | 2320 | 2515 | yes |  |
| 8x8 | confidential signer | full+2tree | yes | 3473 | 27 | 3 | - | 2325 | 2519 | yes |  |
| 8x8 | confidential signer | full+data+ring | yes | 3980 | 26 | 3 | - | 2865 | 3044 | yes |  |
| 8x8 | confidential signer | full+data+program | yes | 3723 | 23 | 2 | - | 2605 | 2784 | yes |  |
| 8x8 | confidential signer | full+data+ring+program | yes | 3947 | 23 | 2 | - | 2829 | 3008 | yes |  |
| 8x8 | confidential in-proof | min | yes | 2871 | 16 | 1 | - | 2225 | 2463 | yes |  |
| 8x8 | confidential in-proof | full | yes | 3434 | 25 | 2 | 6 | 2416 | 2612 | yes |  |
| 8x8 | confidential in-proof | full+spl | yes | 3802 | 32 | 2 | - | 2548 | 2616 | yes |  |
| 8x8 | confidential in-proof | full+data | yes | 3755 | 25 | 2 | - | 2737 | 2917 | yes |  |
| 8x8 | confidential in-proof | full+reply | yes | 3434 | 25 | 2 | - | 2416 | 2612 | yes |  |
| 8x8 | confidential in-proof | full+2tree | yes | 3472 | 26 | 2 | - | 2421 | 2616 | yes |  |
| 8x8 | confidential in-proof | full+data+ring | yes | 3979 | 25 | 2 | - | 2961 | 3141 | yes |  |
| 8x8 | anonymous hidden | min | yes | 2903 | 16 | 1 | - | 2257 | 2535 | yes |  |
| 8x8 | anonymous hidden | full | yes | 3370 | 22 | 2 | 7 | 2448 | 2684 | yes |  |
| 8x8 | anonymous hidden | full+spl | yes | 3738 | 29 | 2 | - | 2580 | 2688 | yes |  |
| 8x8 | anonymous hidden | full+data | yes | 3691 | 22 | 2 | - | 2769 | 2989 | yes |  |
| 8x8 | anonymous hidden | full+reply | yes | 4051 | 22 | 2 | - | 3129 | 3363 | yes |  |
| 8x8 | anonymous hidden | full+2tree | yes | 3408 | 23 | 2 | - | 2453 | 2688 | yes |  |
| 8x8 | anonymous hidden | full+data+ring | yes | 3915 | 22 | 2 | - | 2993 | 3213 | yes |  |
| 8x8 | anonymous public owners | min | yes | 2840 | 16 | 1 | - | 2193 | 2438 | yes |  |
| 8x8 | anonymous public owners | full | yes | 3499 | 26 | 3 | 7 | 2384 | 2587 | yes |  |
| 8x8 | anonymous public owners | full+spl | yes | 3867 | 33 | 3 | - | 2516 | 2591 | yes |  |
| 8x8 | anonymous public owners | full+data | yes | 3820 | 26 | 3 | - | 2705 | 2892 | yes |  |
| 8x8 | anonymous public owners | full+reply | yes | 4180 | 26 | 3 | - | 3065 | 3266 | no | 84 B |
| 8x8 | anonymous public owners | full+2tree | yes | 3537 | 27 | 3 | - | 2389 | 2591 | yes |  |
| 8x8 | anonymous public owners | full+data+ring | yes | 4044 | 26 | 3 | - | 2929 | 3116 | yes |  |
| 8x8 | anonymous public owners | full+data+program | yes | 3787 | 23 | 2 | - | 2669 | 2856 | yes |  |
| 8x8 | anonymous public owners | full+data+ring+program | yes | 4011 | 23 | 2 | - | 2893 | 3080 | yes |  |
| 8x8 | confidential entry | min | yes | 3872 | 16 | 1 | - | 3225 | 3462 | yes |  |
| 8x8 | confidential entry | full | yes | 4531 | 26 | 3 | none | 3416 | 3611 | no | 435 B |
| 8x8 | confidential entry | full+spl | yes | 4899 | 33 | 3 | - | 3548 | 3615 | no | 803 B |
| 8x8 | confidential entry | full+data | yes | 4852 | 26 | 3 | - | 3737 | 3916 | no | 756 B |
| 8x8 | confidential entry | full+reply | yes | 4531 | 26 | 3 | - | 3416 | 3611 | no | 435 B |
| 8x8 | confidential entry | full+2tree | yes | 4569 | 27 | 3 | - | 3421 | 3615 | no | 473 B |
| 8x8 | confidential entry | full+data+ring | yes | 5076 | 26 | 3 | - | 3961 | 4140 | no | 980 B |
| 8x8 | anonymous entry | min | yes | 4031 | 16 | 1 | - | 3385 | 3631 | yes |  |
| 8x8 | anonymous entry | full | yes | 4498 | 22 | 2 | none | 3576 | 3780 | no | 402 B |
| 8x8 | anonymous entry | full+spl | yes | 4866 | 29 | 2 | - | 3708 | 3784 | no | 770 B |
| 8x8 | anonymous entry | full+data | yes | 4819 | 22 | 2 | - | 3897 | 4085 | no | 723 B |
| 8x8 | anonymous entry | full+reply | yes | 5179 | 22 | 2 | - | 4257 | 4459 | no | 1083 B |
| 8x8 | anonymous entry | full+2tree | yes | 4536 | 23 | 2 | - | 3581 | 3784 | no | 440 B |
| 8x8 | anonymous entry | full+data+ring | yes | 5043 | 22 | 2 | - | 4121 | 4309 | no | 947 B |
| 8x8 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 8x8 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 8x16 | confidential signer | min | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential signer | full | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential signer | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential signer | full+data | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential signer | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential signer | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential signer | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential signer | full+data+program | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential signer | full+data+ring+program | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential in-proof | min | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential in-proof | full | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential in-proof | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential in-proof | full+data | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential in-proof | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential in-proof | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential in-proof | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous hidden | min | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous hidden | full | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous hidden | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous hidden | full+data | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous hidden | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous hidden | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous hidden | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous public owners | min | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous public owners | full | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous public owners | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous public owners | full+data | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous public owners | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous public owners | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous public owners | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous public owners | full+data+program | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous public owners | full+data+ring+program | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 8x16 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 8x16 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 12x2 | confidential signer | min | yes | 1974 | 20 | 1 | - | 1195 | 1220 | yes |  |
| 12x2 | confidential signer | full | yes | 2633 | 30 | 3 | 10 | 1386 | 1369 | yes |  |
| 12x2 | confidential signer | full+spl | yes | 3001 | 37 | 3 | - | 1518 | 1373 | yes |  |
| 12x2 | confidential signer | full+data | yes | 2738 | 30 | 3 | - | 1491 | 1470 | yes |  |
| 12x2 | confidential signer | full+reply | yes | 2633 | 30 | 3 | - | 1386 | 1369 | yes |  |
| 12x2 | confidential signer | full+2tree | yes | 2672 | 31 | 3 | - | 1392 | 1373 | yes |  |
| 12x2 | confidential signer | full+data+ring | yes | 2770 | 30 | 3 | - | 1523 | 1502 | yes |  |
| 12x2 | confidential signer | full+data+program | yes | 2705 | 27 | 2 | - | 1455 | 1434 | yes |  |
| 12x2 | confidential signer | full+data+ring+program | yes | 2737 | 27 | 2 | - | 1487 | 1466 | yes |  |
| 12x2 | confidential in-proof | min | yes | 2069 | 20 | 1 | - | 1291 | 1317 | yes |  |
| 12x2 | confidential in-proof | full | yes | 2632 | 29 | 2 | 10 | 1482 | 1466 | yes |  |
| 12x2 | confidential in-proof | full+spl | yes | 3000 | 36 | 2 | - | 1614 | 1470 | yes |  |
| 12x2 | confidential in-proof | full+data | yes | 2737 | 29 | 2 | - | 1587 | 1567 | yes |  |
| 12x2 | confidential in-proof | full+reply | yes | 2632 | 29 | 2 | - | 1482 | 1466 | yes |  |
| 12x2 | confidential in-proof | full+2tree | yes | 2671 | 30 | 2 | - | 1488 | 1470 | yes |  |
| 12x2 | confidential in-proof | full+data+ring | yes | 2769 | 29 | 2 | - | 1619 | 1599 | yes |  |
| 12x2 | anonymous hidden | min | yes | 2101 | 20 | 1 | - | 1323 | 1389 | yes |  |
| 12x2 | anonymous hidden | full | yes | 2568 | 26 | 2 | 10 | 1514 | 1538 | yes |  |
| 12x2 | anonymous hidden | full+spl | yes | 2936 | 33 | 2 | - | 1646 | 1542 | yes |  |
| 12x2 | anonymous hidden | full+data | yes | 2673 | 26 | 2 | - | 1619 | 1639 | yes |  |
| 12x2 | anonymous hidden | full+reply | yes | 2667 | 26 | 2 | - | 1613 | 1635 | yes |  |
| 12x2 | anonymous hidden | full+2tree | yes | 2607 | 27 | 2 | - | 1520 | 1542 | yes |  |
| 12x2 | anonymous hidden | full+data+ring | yes | 2705 | 26 | 2 | - | 1651 | 1671 | yes |  |
| 12x2 | anonymous public owners | min | yes | 2038 | 20 | 1 | - | 1259 | 1292 | yes |  |
| 12x2 | anonymous public owners | full | yes | 2697 | 30 | 3 | 10 | 1450 | 1441 | yes |  |
| 12x2 | anonymous public owners | full+spl | yes | 3065 | 37 | 3 | - | 1582 | 1445 | yes |  |
| 12x2 | anonymous public owners | full+data | yes | 2802 | 30 | 3 | - | 1555 | 1542 | yes |  |
| 12x2 | anonymous public owners | full+reply | yes | 2796 | 30 | 3 | - | 1549 | 1538 | yes |  |
| 12x2 | anonymous public owners | full+2tree | yes | 2736 | 31 | 3 | - | 1456 | 1445 | yes |  |
| 12x2 | anonymous public owners | full+data+ring | yes | 2834 | 30 | 3 | - | 1587 | 1574 | yes |  |
| 12x2 | anonymous public owners | full+data+program | yes | 2769 | 27 | 2 | - | 1519 | 1506 | yes |  |
| 12x2 | anonymous public owners | full+data+ring+program | yes | 2801 | 27 | 2 | - | 1551 | 1538 | yes |  |
| 12x2 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 12x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 12x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 12x4 | confidential signer | min | yes | 2328 | 20 | 1 | - | 1549 | 1646 | yes |  |
| 12x4 | confidential signer | full | yes | 2987 | 30 | 3 | 10 | 1740 | 1795 | yes |  |
| 12x4 | confidential signer | full+spl | yes | 3355 | 37 | 3 | - | 1872 | 1799 | yes |  |
| 12x4 | confidential signer | full+data | yes | 3164 | 30 | 3 | - | 1917 | 1964 | yes |  |
| 12x4 | confidential signer | full+reply | yes | 2987 | 30 | 3 | - | 1740 | 1795 | yes |  |
| 12x4 | confidential signer | full+2tree | yes | 3026 | 31 | 3 | - | 1746 | 1799 | yes |  |
| 12x4 | confidential signer | full+data+ring | yes | 3260 | 30 | 3 | - | 2013 | 2060 | yes |  |
| 12x4 | confidential signer | full+data+program | yes | 3131 | 27 | 2 | - | 1881 | 1928 | yes |  |
| 12x4 | confidential signer | full+data+ring+program | yes | 3227 | 27 | 2 | - | 1977 | 2024 | yes |  |
| 12x4 | confidential in-proof | min | yes | 2423 | 20 | 1 | - | 1645 | 1743 | yes |  |
| 12x4 | confidential in-proof | full | yes | 2986 | 29 | 2 | 10 | 1836 | 1892 | yes |  |
| 12x4 | confidential in-proof | full+spl | yes | 3354 | 36 | 2 | - | 1968 | 1896 | yes |  |
| 12x4 | confidential in-proof | full+data | yes | 3163 | 29 | 2 | - | 2013 | 2061 | yes |  |
| 12x4 | confidential in-proof | full+reply | yes | 2986 | 29 | 2 | - | 1836 | 1892 | yes |  |
| 12x4 | confidential in-proof | full+2tree | yes | 3025 | 30 | 2 | - | 1842 | 1896 | yes |  |
| 12x4 | confidential in-proof | full+data+ring | yes | 3259 | 29 | 2 | - | 2109 | 2157 | yes |  |
| 12x4 | anonymous hidden | min | yes | 2455 | 20 | 1 | - | 1677 | 1815 | yes |  |
| 12x4 | anonymous hidden | full | yes | 2922 | 26 | 2 | 10 | 1868 | 1964 | yes |  |
| 12x4 | anonymous hidden | full+spl | yes | 3290 | 33 | 2 | - | 2000 | 1968 | yes |  |
| 12x4 | anonymous hidden | full+data | yes | 3099 | 26 | 2 | - | 2045 | 2133 | yes |  |
| 12x4 | anonymous hidden | full+reply | yes | 3215 | 26 | 2 | - | 2161 | 2255 | yes |  |
| 12x4 | anonymous hidden | full+2tree | yes | 2961 | 27 | 2 | - | 1874 | 1968 | yes |  |
| 12x4 | anonymous hidden | full+data+ring | yes | 3195 | 26 | 2 | - | 2141 | 2229 | yes |  |
| 12x4 | anonymous public owners | min | yes | 2392 | 20 | 1 | - | 1613 | 1718 | yes |  |
| 12x4 | anonymous public owners | full | yes | 3051 | 30 | 3 | 10 | 1804 | 1867 | yes |  |
| 12x4 | anonymous public owners | full+spl | yes | 3419 | 37 | 3 | - | 1936 | 1871 | yes |  |
| 12x4 | anonymous public owners | full+data | yes | 3228 | 30 | 3 | - | 1981 | 2036 | yes |  |
| 12x4 | anonymous public owners | full+reply | yes | 3344 | 30 | 3 | - | 2097 | 2158 | yes |  |
| 12x4 | anonymous public owners | full+2tree | yes | 3090 | 31 | 3 | - | 1810 | 1871 | yes |  |
| 12x4 | anonymous public owners | full+data+ring | yes | 3324 | 30 | 3 | - | 2077 | 2132 | yes |  |
| 12x4 | anonymous public owners | full+data+program | yes | 3195 | 27 | 2 | - | 1945 | 2000 | yes |  |
| 12x4 | anonymous public owners | full+data+ring+program | yes | 3291 | 27 | 2 | - | 2041 | 2096 | yes |  |
| 12x4 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 12x4 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 12x4 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 12x8 | confidential signer | min | yes | 3036 | 20 | 1 | - | 2257 | 2498 | yes |  |
| 12x8 | confidential signer | full | yes | 3695 | 30 | 3 | 5 | 2448 | 2647 | yes |  |
| 12x8 | confidential signer | full+spl | yes | 4063 | 37 | 3 | - | 2580 | 2651 | yes |  |
| 12x8 | confidential signer | full+data | yes | 4016 | 30 | 3 | - | 2769 | 2952 | yes |  |
| 12x8 | confidential signer | full+reply | yes | 3695 | 30 | 3 | - | 2448 | 2647 | yes |  |
| 12x8 | confidential signer | full+2tree | yes | 3734 | 31 | 3 | - | 2454 | 2651 | yes |  |
| 12x8 | confidential signer | full+data+ring | yes | 4240 | 30 | 3 | - | 2993 | 3176 | no | 144 B |
| 12x8 | confidential signer | full+data+program | yes | 3983 | 27 | 2 | - | 2733 | 2916 | yes |  |
| 12x8 | confidential signer | full+data+ring+program | yes | 4207 | 27 | 2 | - | 2957 | 3140 | no | 111 B |
| 12x8 | confidential in-proof | min | yes | 3131 | 20 | 1 | - | 2353 | 2595 | yes |  |
| 12x8 | confidential in-proof | full | yes | 3694 | 29 | 2 | 4 | 2544 | 2744 | yes |  |
| 12x8 | confidential in-proof | full+spl | yes | 4062 | 36 | 2 | - | 2676 | 2748 | yes |  |
| 12x8 | confidential in-proof | full+data | yes | 4015 | 29 | 2 | - | 2865 | 3049 | yes |  |
| 12x8 | confidential in-proof | full+reply | yes | 3694 | 29 | 2 | - | 2544 | 2744 | yes |  |
| 12x8 | confidential in-proof | full+2tree | yes | 3733 | 30 | 2 | - | 2550 | 2748 | yes |  |
| 12x8 | confidential in-proof | full+data+ring | yes | 4239 | 29 | 2 | - | 3089 | 3273 | no | 143 B |
| 12x8 | anonymous hidden | min | yes | 3163 | 20 | 1 | - | 2385 | 2667 | yes |  |
| 12x8 | anonymous hidden | full | yes | 3630 | 26 | 2 | 4 | 2576 | 2816 | yes |  |
| 12x8 | anonymous hidden | full+spl | yes | 3998 | 33 | 2 | - | 2708 | 2820 | yes |  |
| 12x8 | anonymous hidden | full+data | yes | 3951 | 26 | 2 | - | 2897 | 3121 | yes |  |
| 12x8 | anonymous hidden | full+reply | yes | 4311 | 26 | 2 | - | 3257 | 3495 | no | 215 B |
| 12x8 | anonymous hidden | full+2tree | yes | 3669 | 27 | 2 | - | 2582 | 2820 | yes |  |
| 12x8 | anonymous hidden | full+data+ring | yes | 4175 | 26 | 2 | - | 3121 | 3345 | no | 79 B |
| 12x8 | anonymous public owners | min | yes | 3100 | 20 | 1 | - | 2321 | 2570 | yes |  |
| 12x8 | anonymous public owners | full | yes | 3759 | 30 | 3 | 4 | 2512 | 2719 | yes |  |
| 12x8 | anonymous public owners | full+spl | yes | 4127 | 37 | 3 | - | 2644 | 2723 | no | 31 B |
| 12x8 | anonymous public owners | full+data | yes | 4080 | 30 | 3 | - | 2833 | 3024 | yes |  |
| 12x8 | anonymous public owners | full+reply | yes | 4440 | 30 | 3 | - | 3193 | 3398 | no | 344 B |
| 12x8 | anonymous public owners | full+2tree | yes | 3798 | 31 | 3 | - | 2518 | 2723 | yes |  |
| 12x8 | anonymous public owners | full+data+ring | yes | 4304 | 30 | 3 | - | 3057 | 3248 | no | 208 B |
| 12x8 | anonymous public owners | full+data+program | yes | 4047 | 27 | 2 | - | 2797 | 2988 | yes |  |
| 12x8 | anonymous public owners | full+data+ring+program | yes | 4271 | 27 | 2 | - | 3021 | 3212 | no | 175 B |
| 12x8 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 12x8 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 12x8 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 16x2 | confidential signer | min | yes | 2234 | 24 | 1 | - | 1323 | 1352 | yes |  |
| 16x2 | confidential signer | full | yes | 2893 | 34 | 3 | 10 | 1514 | 1501 | yes |  |
| 16x2 | confidential signer | full+spl | yes | 3261 | 41 | 3 | - | 1646 | 1505 | yes |  |
| 16x2 | confidential signer | full+data | yes | 2998 | 34 | 3 | - | 1619 | 1602 | yes |  |
| 16x2 | confidential signer | full+reply | yes | 2893 | 34 | 3 | - | 1514 | 1501 | yes |  |
| 16x2 | confidential signer | full+2tree | yes | 2932 | 35 | 3 | - | 1520 | 1505 | yes |  |
| 16x2 | confidential signer | full+data+ring | yes | 3030 | 34 | 3 | - | 1651 | 1634 | yes |  |
| 16x2 | confidential signer | full+data+program | yes | 2965 | 31 | 2 | - | 1583 | 1566 | yes |  |
| 16x2 | confidential signer | full+data+ring+program | yes | 2997 | 31 | 2 | - | 1615 | 1598 | yes |  |
| 16x2 | confidential in-proof | min | yes | 2329 | 24 | 1 | - | 1419 | 1449 | yes |  |
| 16x2 | confidential in-proof | full | yes | 2892 | 33 | 2 | 10 | 1610 | 1598 | yes |  |
| 16x2 | confidential in-proof | full+spl | yes | 3260 | 40 | 2 | - | 1742 | 1602 | yes |  |
| 16x2 | confidential in-proof | full+data | yes | 2997 | 33 | 2 | - | 1715 | 1699 | yes |  |
| 16x2 | confidential in-proof | full+reply | yes | 2892 | 33 | 2 | - | 1610 | 1598 | yes |  |
| 16x2 | confidential in-proof | full+2tree | yes | 2931 | 34 | 2 | - | 1616 | 1602 | yes |  |
| 16x2 | confidential in-proof | full+data+ring | yes | 3029 | 33 | 2 | - | 1747 | 1731 | yes |  |
| 16x2 | anonymous hidden | min | yes | 2361 | 24 | 1 | - | 1451 | 1521 | yes |  |
| 16x2 | anonymous hidden | full | yes | 2828 | 30 | 2 | 10 | 1642 | 1670 | yes |  |
| 16x2 | anonymous hidden | full+spl | yes | 3196 | 37 | 2 | - | 1774 | 1674 | yes |  |
| 16x2 | anonymous hidden | full+data | yes | 2933 | 30 | 2 | - | 1747 | 1771 | yes |  |
| 16x2 | anonymous hidden | full+reply | yes | 2927 | 30 | 2 | - | 1741 | 1767 | yes |  |
| 16x2 | anonymous hidden | full+2tree | yes | 2867 | 31 | 2 | - | 1648 | 1674 | yes |  |
| 16x2 | anonymous hidden | full+data+ring | yes | 2965 | 30 | 2 | - | 1779 | 1803 | yes |  |
| 16x2 | anonymous public owners | min | yes | 2298 | 24 | 1 | - | 1387 | 1424 | yes |  |
| 16x2 | anonymous public owners | full | yes | 2957 | 34 | 3 | 10 | 1578 | 1573 | yes |  |
| 16x2 | anonymous public owners | full+spl | yes | 3325 | 41 | 3 | - | 1710 | 1577 | yes |  |
| 16x2 | anonymous public owners | full+data | yes | 3062 | 34 | 3 | - | 1683 | 1674 | yes |  |
| 16x2 | anonymous public owners | full+reply | yes | 3056 | 34 | 3 | - | 1677 | 1670 | yes |  |
| 16x2 | anonymous public owners | full+2tree | yes | 2996 | 35 | 3 | - | 1584 | 1577 | yes |  |
| 16x2 | anonymous public owners | full+data+ring | yes | 3094 | 34 | 3 | - | 1715 | 1706 | yes |  |
| 16x2 | anonymous public owners | full+data+program | yes | 3029 | 31 | 2 | - | 1647 | 1638 | yes |  |
| 16x2 | anonymous public owners | full+data+ring+program | yes | 3061 | 31 | 2 | - | 1679 | 1670 | yes |  |
| 16x2 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 16x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 16x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 16x4 | confidential signer | min | yes | 2588 | 24 | 1 | - | 1677 | 1778 | yes |  |
| 16x4 | confidential signer | full | yes | 3247 | 34 | 3 | 9 | 1868 | 1927 | yes |  |
| 16x4 | confidential signer | full+spl | yes | 3615 | 41 | 3 | - | 2000 | 1931 | yes |  |
| 16x4 | confidential signer | full+data | yes | 3424 | 34 | 3 | - | 2045 | 2096 | yes |  |
| 16x4 | confidential signer | full+reply | yes | 3247 | 34 | 3 | - | 1868 | 1927 | yes |  |
| 16x4 | confidential signer | full+2tree | yes | 3286 | 35 | 3 | - | 1874 | 1931 | yes |  |
| 16x4 | confidential signer | full+data+ring | yes | 3520 | 34 | 3 | - | 2141 | 2192 | yes |  |
| 16x4 | confidential signer | full+data+program | yes | 3391 | 31 | 2 | - | 2009 | 2060 | yes |  |
| 16x4 | confidential signer | full+data+ring+program | yes | 3487 | 31 | 2 | - | 2105 | 2156 | yes |  |
| 16x4 | confidential in-proof | min | yes | 2683 | 24 | 1 | - | 1773 | 1875 | yes |  |
| 16x4 | confidential in-proof | full | yes | 3246 | 33 | 2 | 8 | 1964 | 2024 | yes |  |
| 16x4 | confidential in-proof | full+spl | yes | 3614 | 40 | 2 | - | 2096 | 2028 | yes |  |
| 16x4 | confidential in-proof | full+data | yes | 3423 | 33 | 2 | - | 2141 | 2193 | yes |  |
| 16x4 | confidential in-proof | full+reply | yes | 3246 | 33 | 2 | - | 1964 | 2024 | yes |  |
| 16x4 | confidential in-proof | full+2tree | yes | 3285 | 34 | 2 | - | 1970 | 2028 | yes |  |
| 16x4 | confidential in-proof | full+data+ring | yes | 3519 | 33 | 2 | - | 2237 | 2289 | yes |  |
| 16x4 | anonymous hidden | min | yes | 2715 | 24 | 1 | - | 1805 | 1947 | yes |  |
| 16x4 | anonymous hidden | full | yes | 3182 | 30 | 2 | 9 | 1996 | 2096 | yes |  |
| 16x4 | anonymous hidden | full+spl | yes | 3550 | 37 | 2 | - | 2128 | 2100 | yes |  |
| 16x4 | anonymous hidden | full+data | yes | 3359 | 30 | 2 | - | 2173 | 2265 | yes |  |
| 16x4 | anonymous hidden | full+reply | yes | 3475 | 30 | 2 | - | 2289 | 2387 | yes |  |
| 16x4 | anonymous hidden | full+2tree | yes | 3221 | 31 | 2 | - | 2002 | 2100 | yes |  |
| 16x4 | anonymous hidden | full+data+ring | yes | 3455 | 30 | 2 | - | 2269 | 2361 | yes |  |
| 16x4 | anonymous public owners | min | yes | 2652 | 24 | 1 | - | 1741 | 1850 | yes |  |
| 16x4 | anonymous public owners | full | yes | 3311 | 34 | 3 | 9 | 1932 | 1999 | yes |  |
| 16x4 | anonymous public owners | full+spl | yes | 3679 | 41 | 3 | - | 2064 | 2003 | yes |  |
| 16x4 | anonymous public owners | full+data | yes | 3488 | 34 | 3 | - | 2109 | 2168 | yes |  |
| 16x4 | anonymous public owners | full+reply | yes | 3604 | 34 | 3 | - | 2225 | 2290 | yes |  |
| 16x4 | anonymous public owners | full+2tree | yes | 3350 | 35 | 3 | - | 1938 | 2003 | yes |  |
| 16x4 | anonymous public owners | full+data+ring | yes | 3584 | 34 | 3 | - | 2205 | 2264 | yes |  |
| 16x4 | anonymous public owners | full+data+program | yes | 3455 | 31 | 2 | - | 2073 | 2132 | yes |  |
| 16x4 | anonymous public owners | full+data+ring+program | yes | 3551 | 31 | 2 | - | 2169 | 2228 | yes |  |
| 16x4 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 16x4 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 16x4 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 16x8 | confidential signer | min | yes | 3296 | 24 | 1 | - | 2385 | 2630 | yes |  |
| 16x8 | confidential signer | full | yes | 3955 | 34 | 3 | 2 | 2576 | 2779 | yes |  |
| 16x8 | confidential signer | full+spl | yes | 4323 | 41 | 3 | - | 2708 | 2783 | no | 227 B |
| 16x8 | confidential signer | full+data | yes | 4276 | 34 | 3 | - | 2897 | 3084 | no | 180 B |
| 16x8 | confidential signer | full+reply | yes | 3955 | 34 | 3 | - | 2576 | 2779 | yes |  |
| 16x8 | confidential signer | full+2tree | yes | 3994 | 35 | 3 | - | 2582 | 2783 | yes |  |
| 16x8 | confidential signer | full+data+ring | yes | 4500 | 34 | 3 | - | 3121 | 3308 | no | 404 B |
| 16x8 | confidential signer | full+data+program | yes | 4243 | 31 | 2 | - | 2861 | 3048 | no | 147 B |
| 16x8 | confidential signer | full+data+ring+program | yes | 4467 | 31 | 2 | - | 3085 | 3272 | no | 371 B |
| 16x8 | confidential in-proof | min | yes | 3391 | 24 | 1 | - | 2481 | 2727 | yes |  |
| 16x8 | confidential in-proof | full | yes | 3954 | 33 | 2 | 1 | 2672 | 2876 | yes |  |
| 16x8 | confidential in-proof | full+spl | yes | 4322 | 40 | 2 | - | 2804 | 2880 | no | 226 B |
| 16x8 | confidential in-proof | full+data | yes | 4275 | 33 | 2 | - | 2993 | 3181 | no | 179 B |
| 16x8 | confidential in-proof | full+reply | yes | 3954 | 33 | 2 | - | 2672 | 2876 | yes |  |
| 16x8 | confidential in-proof | full+2tree | yes | 3993 | 34 | 2 | - | 2678 | 2880 | yes |  |
| 16x8 | confidential in-proof | full+data+ring | yes | 4499 | 33 | 2 | - | 3217 | 3405 | no | 403 B |
| 16x8 | anonymous hidden | min | yes | 3423 | 24 | 1 | - | 2513 | 2799 | yes |  |
| 16x8 | anonymous hidden | full | yes | 3890 | 30 | 2 | 2 | 2704 | 2948 | yes |  |
| 16x8 | anonymous hidden | full+spl | yes | 4258 | 37 | 2 | - | 2836 | 2952 | no | 162 B |
| 16x8 | anonymous hidden | full+data | yes | 4211 | 30 | 2 | - | 3025 | 3253 | no | 115 B |
| 16x8 | anonymous hidden | full+reply | yes | 4571 | 30 | 2 | - | 3385 | 3627 | no | 475 B |
| 16x8 | anonymous hidden | full+2tree | yes | 3929 | 31 | 2 | - | 2710 | 2952 | yes |  |
| 16x8 | anonymous hidden | full+data+ring | yes | 4435 | 30 | 2 | - | 3249 | 3477 | no | 339 B |
| 16x8 | anonymous public owners | min | yes | 3360 | 24 | 1 | - | 2449 | 2702 | yes |  |
| 16x8 | anonymous public owners | full | yes | 4019 | 34 | 3 | 1 | 2640 | 2851 | yes |  |
| 16x8 | anonymous public owners | full+spl | yes | 4387 | 41 | 3 | - | 2772 | 2855 | no | 291 B |
| 16x8 | anonymous public owners | full+data | yes | 4340 | 34 | 3 | - | 2961 | 3156 | no | 244 B |
| 16x8 | anonymous public owners | full+reply | yes | 4700 | 34 | 3 | - | 3321 | 3530 | no | 604 B |
| 16x8 | anonymous public owners | full+2tree | yes | 4058 | 35 | 3 | - | 2646 | 2855 | yes |  |
| 16x8 | anonymous public owners | full+data+ring | yes | 4564 | 34 | 3 | - | 3185 | 3380 | no | 468 B |
| 16x8 | anonymous public owners | full+data+program | yes | 4307 | 31 | 2 | - | 2925 | 3120 | no | 211 B |
| 16x8 | anonymous public owners | full+data+ring+program | yes | 4531 | 31 | 2 | - | 3149 | 3344 | no | 435 B |
| 16x8 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 16x8 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 16x8 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 24x2 | confidential signer | min | yes | 2754 | 32 | 1 | - | 1579 | 1616 | yes |  |
| 24x2 | confidential signer | full | yes | 3413 | 42 | 3 | 8 | 1770 | 1765 | yes |  |
| 24x2 | confidential signer | full+spl | yes | 3781 | 49 | 3 | - | 1902 | 1769 | yes |  |
| 24x2 | confidential signer | full+data | yes | 3518 | 42 | 3 | - | 1875 | 1866 | yes |  |
| 24x2 | confidential signer | full+reply | yes | 3413 | 42 | 3 | - | 1770 | 1765 | yes |  |
| 24x2 | confidential signer | full+2tree | yes | 3453 | 43 | 3 | - | 1777 | 1769 | yes |  |
| 24x2 | confidential signer | full+data+ring | yes | 3550 | 42 | 3 | - | 1907 | 1898 | yes |  |
| 24x2 | confidential signer | full+data+program | yes | 3485 | 39 | 2 | - | 1839 | 1830 | yes |  |
| 24x2 | confidential signer | full+data+ring+program | yes | 3517 | 39 | 2 | - | 1871 | 1862 | yes |  |
| 24x2 | confidential in-proof | min | yes | 2849 | 32 | 1 | - | 1675 | 1713 | yes |  |
| 24x2 | confidential in-proof | full | yes | 3412 | 41 | 2 | 7 | 1866 | 1862 | yes |  |
| 24x2 | confidential in-proof | full+spl | yes | 3780 | 48 | 2 | - | 1998 | 1866 | yes |  |
| 24x2 | confidential in-proof | full+data | yes | 3517 | 41 | 2 | - | 1971 | 1963 | yes |  |
| 24x2 | confidential in-proof | full+reply | yes | 3412 | 41 | 2 | - | 1866 | 1862 | yes |  |
| 24x2 | confidential in-proof | full+2tree | yes | 3452 | 42 | 2 | - | 1873 | 1866 | yes |  |
| 24x2 | confidential in-proof | full+data+ring | yes | 3549 | 41 | 2 | - | 2003 | 1995 | yes |  |
| 24x2 | anonymous hidden | min | yes | 2881 | 32 | 1 | - | 1707 | 1785 | yes |  |
| 24x2 | anonymous hidden | full | yes | 3348 | 38 | 2 | 7 | 1898 | 1934 | yes |  |
| 24x2 | anonymous hidden | full+spl | yes | 3716 | 45 | 2 | - | 2030 | 1938 | yes |  |
| 24x2 | anonymous hidden | full+data | yes | 3453 | 38 | 2 | - | 2003 | 2035 | yes |  |
| 24x2 | anonymous hidden | full+reply | yes | 3447 | 38 | 2 | - | 1997 | 2031 | yes |  |
| 24x2 | anonymous hidden | full+2tree | yes | 3388 | 39 | 2 | - | 1905 | 1938 | yes |  |
| 24x2 | anonymous hidden | full+data+ring | yes | 3485 | 38 | 2 | - | 2035 | 2067 | yes |  |
| 24x2 | anonymous public owners | min | yes | 2818 | 32 | 1 | - | 1643 | 1688 | yes |  |
| 24x2 | anonymous public owners | full | yes | 3477 | 42 | 3 | 7 | 1834 | 1837 | yes |  |
| 24x2 | anonymous public owners | full+spl | yes | 3845 | 49 | 3 | - | 1966 | 1841 | yes |  |
| 24x2 | anonymous public owners | full+data | yes | 3582 | 42 | 3 | - | 1939 | 1938 | yes |  |
| 24x2 | anonymous public owners | full+reply | yes | 3576 | 42 | 3 | - | 1933 | 1934 | yes |  |
| 24x2 | anonymous public owners | full+2tree | yes | 3517 | 43 | 3 | - | 1841 | 1841 | yes |  |
| 24x2 | anonymous public owners | full+data+ring | yes | 3614 | 42 | 3 | - | 1971 | 1970 | yes |  |
| 24x2 | anonymous public owners | full+data+program | yes | 3549 | 39 | 2 | - | 1903 | 1902 | yes |  |
| 24x2 | anonymous public owners | full+data+ring+program | yes | 3581 | 39 | 2 | - | 1935 | 1934 | yes |  |
| 24x2 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 24x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 24x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 24x4 | confidential signer | min | yes | 3108 | 32 | 1 | - | 1933 | 2042 | yes |  |
| 24x4 | confidential signer | full | yes | 3767 | 42 | 3 | 4 | 2124 | 2191 | yes |  |
| 24x4 | confidential signer | full+spl | yes | 4135 | 49 | 3 | - | 2256 | 2195 | no | 39 B |
| 24x4 | confidential signer | full+data | yes | 3944 | 42 | 3 | - | 2301 | 2360 | yes |  |
| 24x4 | confidential signer | full+reply | yes | 3767 | 42 | 3 | - | 2124 | 2191 | yes |  |
| 24x4 | confidential signer | full+2tree | yes | 3807 | 43 | 3 | - | 2131 | 2195 | yes |  |
| 24x4 | confidential signer | full+data+ring | yes | 4040 | 42 | 3 | - | 2397 | 2456 | yes |  |
| 24x4 | confidential signer | full+data+program | yes | 3911 | 39 | 2 | - | 2265 | 2324 | yes |  |
| 24x4 | confidential signer | full+data+ring+program | yes | 4007 | 39 | 2 | - | 2361 | 2420 | yes |  |
| 24x4 | confidential in-proof | min | yes | 3203 | 32 | 1 | - | 2029 | 2139 | yes |  |
| 24x4 | confidential in-proof | full | yes | 3766 | 41 | 2 | 3 | 2220 | 2288 | yes |  |
| 24x4 | confidential in-proof | full+spl | yes | 4134 | 48 | 2 | - | 2352 | 2292 | no | 38 B |
| 24x4 | confidential in-proof | full+data | yes | 3943 | 41 | 2 | - | 2397 | 2457 | yes |  |
| 24x4 | confidential in-proof | full+reply | yes | 3766 | 41 | 2 | - | 2220 | 2288 | yes |  |
| 24x4 | confidential in-proof | full+2tree | yes | 3806 | 42 | 2 | - | 2227 | 2292 | yes |  |
| 24x4 | confidential in-proof | full+data+ring | yes | 4039 | 41 | 2 | - | 2493 | 2553 | yes |  |
| 24x4 | anonymous hidden | min | yes | 3235 | 32 | 1 | - | 2061 | 2211 | yes |  |
| 24x4 | anonymous hidden | full | yes | 3702 | 38 | 2 | 4 | 2252 | 2360 | yes |  |
| 24x4 | anonymous hidden | full+spl | yes | 4070 | 45 | 2 | - | 2384 | 2364 | yes |  |
| 24x4 | anonymous hidden | full+data | yes | 3879 | 38 | 2 | - | 2429 | 2529 | yes |  |
| 24x4 | anonymous hidden | full+reply | yes | 3995 | 38 | 2 | - | 2545 | 2651 | yes |  |
| 24x4 | anonymous hidden | full+2tree | yes | 3742 | 39 | 2 | - | 2259 | 2364 | yes |  |
| 24x4 | anonymous hidden | full+data+ring | yes | 3975 | 38 | 2 | - | 2525 | 2625 | yes |  |
| 24x4 | anonymous public owners | min | yes | 3172 | 32 | 1 | - | 1997 | 2114 | yes |  |
| 24x4 | anonymous public owners | full | yes | 3831 | 42 | 3 | 3 | 2188 | 2263 | yes |  |
| 24x4 | anonymous public owners | full+spl | yes | 4199 | 49 | 3 | - | 2320 | 2267 | no | 103 B |
| 24x4 | anonymous public owners | full+data | yes | 4008 | 42 | 3 | - | 2365 | 2432 | yes |  |
| 24x4 | anonymous public owners | full+reply | yes | 4124 | 42 | 3 | - | 2481 | 2554 | no | 28 B |
| 24x4 | anonymous public owners | full+2tree | yes | 3871 | 43 | 3 | - | 2195 | 2267 | yes |  |
| 24x4 | anonymous public owners | full+data+ring | yes | 4104 | 42 | 3 | - | 2461 | 2528 | no | 8 B |
| 24x4 | anonymous public owners | full+data+program | yes | 3975 | 39 | 2 | - | 2329 | 2396 | yes |  |
| 24x4 | anonymous public owners | full+data+ring+program | yes | 4071 | 39 | 2 | - | 2425 | 2492 | yes |  |
| 24x4 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 24x4 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 24x4 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 32x2 | confidential signer | min | yes | 3274 | 40 | 1 | - | 1835 | 1880 | yes |  |
| 32x2 | confidential signer | full | yes | 3933 | 50 | 3 | 2 | 2026 | 2029 | yes |  |
| 32x2 | confidential signer | full+spl | yes | 4301 | 57 | 3 | - | 2158 | 2033 | no | 205 B |
| 32x2 | confidential signer | full+data | yes | 4038 | 50 | 3 | - | 2131 | 2130 | yes |  |
| 32x2 | confidential signer | full+reply | yes | 3933 | 50 | 3 | - | 2026 | 2029 | yes |  |
| 32x2 | confidential signer | full+2tree | yes | 3974 | 51 | 3 | - | 2034 | 2033 | yes |  |
| 32x2 | confidential signer | full+data+ring | yes | 4070 | 50 | 3 | - | 2163 | 2162 | yes |  |
| 32x2 | confidential signer | full+data+program | yes | 4005 | 47 | 2 | - | 2095 | 2094 | yes |  |
| 32x2 | confidential signer | full+data+ring+program | yes | 4037 | 47 | 2 | - | 2127 | 2126 | yes |  |
| 32x2 | confidential in-proof | min | yes | 3369 | 40 | 1 | - | 1931 | 1977 | yes |  |
| 32x2 | confidential in-proof | full | yes | 3932 | 49 | 2 | 1 | 2122 | 2126 | yes |  |
| 32x2 | confidential in-proof | full+spl | yes | 4300 | 56 | 2 | - | 2254 | 2130 | no | 204 B |
| 32x2 | confidential in-proof | full+data | yes | 4037 | 49 | 2 | - | 2227 | 2227 | yes |  |
| 32x2 | confidential in-proof | full+reply | yes | 3932 | 49 | 2 | - | 2122 | 2126 | yes |  |
| 32x2 | confidential in-proof | full+2tree | yes | 3973 | 50 | 2 | - | 2130 | 2130 | yes |  |
| 32x2 | confidential in-proof | full+data+ring | yes | 4069 | 49 | 2 | - | 2259 | 2259 | yes |  |
| 32x2 | anonymous hidden | min | yes | 3401 | 40 | 1 | - | 1963 | 2049 | yes |  |
| 32x2 | anonymous hidden | full | yes | 3868 | 46 | 2 | 2 | 2154 | 2198 | yes |  |
| 32x2 | anonymous hidden | full+spl | yes | 4236 | 53 | 2 | - | 2286 | 2202 | no | 140 B |
| 32x2 | anonymous hidden | full+data | yes | 3973 | 46 | 2 | - | 2259 | 2299 | yes |  |
| 32x2 | anonymous hidden | full+reply | yes | 3967 | 46 | 2 | - | 2253 | 2295 | yes |  |
| 32x2 | anonymous hidden | full+2tree | yes | 3909 | 47 | 2 | - | 2162 | 2202 | yes |  |
| 32x2 | anonymous hidden | full+data+ring | yes | 4005 | 46 | 2 | - | 2291 | 2331 | yes |  |
| 32x2 | anonymous public owners | min | yes | 3338 | 40 | 1 | - | 1899 | 1952 | yes |  |
| 32x2 | anonymous public owners | full | yes | 3997 | 50 | 3 | 2 | 2090 | 2101 | yes |  |
| 32x2 | anonymous public owners | full+spl | yes | 4365 | 57 | 3 | - | 2222 | 2105 | no | 269 B |
| 32x2 | anonymous public owners | full+data | yes | 4102 | 50 | 3 | - | 2195 | 2202 | no | 6 B |
| 32x2 | anonymous public owners | full+reply | yes | 4096 | 50 | 3 | - | 2189 | 2198 | yes |  |
| 32x2 | anonymous public owners | full+2tree | yes | 4038 | 51 | 3 | - | 2098 | 2105 | yes |  |
| 32x2 | anonymous public owners | full+data+ring | yes | 4134 | 50 | 3 | - | 2227 | 2234 | no | 38 B |
| 32x2 | anonymous public owners | full+data+program | yes | 4069 | 47 | 2 | - | 2159 | 2166 | yes |  |
| 32x2 | anonymous public owners | full+data+ring+program | yes | 4101 | 47 | 2 | - | 2191 | 2198 | no | 5 B |
| 32x2 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 32x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 32x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 40x2 | confidential signer | min | yes | 3794 | 48 | 1 | - | 2091 | 2144 | yes |  |
| 40x2 | confidential signer | full | yes | 4453 | 58 | 3 | none | 2282 | 2293 | no | 357 B |
| 40x2 | confidential signer | full+spl | yes | 4821 | 65 | 3 | - | 2414 | 2297 | no | 725 B, 1 accounts |
| 40x2 | confidential signer | full+data | yes | 4558 | 58 | 3 | - | 2387 | 2394 | no | 462 B |
| 40x2 | confidential signer | full+reply | yes | 4453 | 58 | 3 | - | 2282 | 2293 | no | 357 B |
| 40x2 | confidential signer | full+2tree | yes | 4495 | 59 | 3 | - | 2291 | 2297 | no | 399 B |
| 40x2 | confidential signer | full+data+ring | yes | 4590 | 58 | 3 | - | 2419 | 2426 | no | 494 B |
| 40x2 | confidential signer | full+data+program | yes | 4525 | 55 | 2 | - | 2351 | 2358 | no | 429 B |
| 40x2 | confidential signer | full+data+ring+program | yes | 4557 | 55 | 2 | - | 2383 | 2390 | no | 461 B |
| 40x2 | confidential in-proof | min | yes | 3889 | 48 | 1 | - | 2187 | 2241 | yes |  |
| 40x2 | confidential in-proof | full | yes | 4452 | 57 | 2 | none | 2378 | 2390 | no | 356 B |
| 40x2 | confidential in-proof | full+spl | yes | 4820 | 64 | 2 | - | 2510 | 2394 | no | 724 B |
| 40x2 | confidential in-proof | full+data | yes | 4557 | 57 | 2 | - | 2483 | 2491 | no | 461 B |
| 40x2 | confidential in-proof | full+reply | yes | 4452 | 57 | 2 | - | 2378 | 2390 | no | 356 B |
| 40x2 | confidential in-proof | full+2tree | yes | 4494 | 58 | 2 | - | 2387 | 2394 | no | 398 B |
| 40x2 | confidential in-proof | full+data+ring | yes | 4589 | 57 | 2 | - | 2515 | 2523 | no | 493 B |
| 40x2 | anonymous hidden | min | yes | 3921 | 48 | 1 | - | 2219 | 2313 | yes |  |
| 40x2 | anonymous hidden | full | yes | 4388 | 54 | 2 | none | 2410 | 2462 | no | 292 B |
| 40x2 | anonymous hidden | full+spl | yes | 4756 | 61 | 2 | - | 2542 | 2466 | no | 660 B |
| 40x2 | anonymous hidden | full+data | yes | 4493 | 54 | 2 | - | 2515 | 2563 | no | 397 B |
| 40x2 | anonymous hidden | full+reply | yes | 4487 | 54 | 2 | - | 2509 | 2559 | no | 391 B |
| 40x2 | anonymous hidden | full+2tree | yes | 4430 | 55 | 2 | - | 2419 | 2466 | no | 334 B |
| 40x2 | anonymous hidden | full+data+ring | yes | 4525 | 54 | 2 | - | 2547 | 2595 | no | 429 B |
| 40x2 | anonymous public owners | min | yes | 3858 | 48 | 1 | - | 2155 | 2216 | yes |  |
| 40x2 | anonymous public owners | full | yes | 4517 | 58 | 3 | none | 2346 | 2365 | no | 421 B |
| 40x2 | anonymous public owners | full+spl | yes | 4885 | 65 | 3 | - | 2478 | 2369 | no | 789 B, 1 accounts |
| 40x2 | anonymous public owners | full+data | yes | 4622 | 58 | 3 | - | 2451 | 2466 | no | 526 B |
| 40x2 | anonymous public owners | full+reply | yes | 4616 | 58 | 3 | - | 2445 | 2462 | no | 520 B |
| 40x2 | anonymous public owners | full+2tree | yes | 4559 | 59 | 3 | - | 2355 | 2369 | no | 463 B |
| 40x2 | anonymous public owners | full+data+ring | yes | 4654 | 58 | 3 | - | 2483 | 2498 | no | 558 B |
| 40x2 | anonymous public owners | full+data+program | yes | 4589 | 55 | 2 | - | 2415 | 2430 | no | 493 B |
| 40x2 | anonymous public owners | full+data+ring+program | yes | 4621 | 55 | 2 | - | 2447 | 2462 | no | 525 B |
| 40x2 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 40x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 40x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 48x2 | confidential signer | min | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential signer | full | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential signer | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential signer | full+data | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential signer | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential signer | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential signer | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential signer | full+data+program | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential signer | full+data+ring+program | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential in-proof | min | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential in-proof | full | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential in-proof | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential in-proof | full+data | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential in-proof | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential in-proof | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential in-proof | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous hidden | min | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous hidden | full | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous hidden | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous hidden | full+data | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous hidden | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous hidden | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous hidden | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous public owners | min | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous public owners | full | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous public owners | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous public owners | full+data | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous public owners | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous public owners | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous public owners | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous public owners | full+data+program | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous public owners | full+data+ring+program | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 48x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 48x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 49x2 | confidential signer | min | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential signer | full | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential signer | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential signer | full+data | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential signer | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential signer | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential signer | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential signer | full+data+program | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential signer | full+data+ring+program | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential in-proof | min | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential in-proof | full | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential in-proof | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential in-proof | full+data | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential in-proof | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential in-proof | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential in-proof | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous hidden | min | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous hidden | full | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous hidden | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous hidden | full+data | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous hidden | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous hidden | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous hidden | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous public owners | min | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous public owners | full | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous public owners | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous public owners | full+data | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous public owners | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous public owners | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous public owners | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous public owners | full+data+program | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous public owners | full+data+ring+program | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential entry | min | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential entry | full | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | confidential entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous entry | min | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous entry | full | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous entry | full+spl | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous entry | full+data | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous entry | full+reply | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous entry | full+2tree | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | anonymous entry | full+data+ring | none | - | - | - | - | - | - | no | catalog key |
| 49x2 | delegate | - | - | - | - | - | - | - | - | no | rail |
| 49x2 | anonymous delegate | - | - | - | - | - | - | - | - | no | rail |
| 8x1 | merge | ring | yes | 1728 | 16 | 1 | - | - | - | yes |  |
| 24x1 | merge | ring | yes | 2768 | 32 | 1 | - | - | - | yes |  |
| 52x1 | merge | ring | yes | 4588 | 60 | 1 | - | - | - | no | 492 B |

Entry capacities at two outputs. Each row is the widest owner run one transaction clears at that leg count, rows = run + a hidden sender. A confidential leg names its own destination, a hidden sender's legs name one. over: what stops the next owner.
| shape | rail | load | legs | leg kind | owner run | rows | accounts | signatures | frame | SPP CPI | fits | over |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 12x2 | confidential entry | min | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 12x2 | confidential entry | full | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 12x2 | confidential entry | full+data+reply | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 12x2 | confidential entry | min | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 12x2 | confidential entry | full | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 12x2 | confidential entry | full+data+reply | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | min | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | full | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | full+data+reply | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | min | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | full | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 12x2 | anonymous entry | full+data+reply | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | min | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | full | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | full+data+reply | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | min | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | full | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 16x2 | confidential entry | full+data+reply | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | min | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | full | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | full+data+reply | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | min | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | full | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 16x2 | anonymous entry | full+data+reply | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | min | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | full | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | full+data+reply | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | min | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | full | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 24x2 | confidential entry | full+data+reply | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | min | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | full | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | full+data+reply | 0 | sol | none | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | min | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | full | 0 | spl | none | - | - | - | - | - | no | catalog key |
| 24x2 | anonymous entry | full+data+reply | 0 | spl | none | - | - | - | - | - | no | catalog key |

Maxima. Load max is full with recipient payloads, every extension but the cache, two SPP input trees, replies on anonymous frames. Each row widens one axis from that load to the largest value one transaction clears. reply: the reply length. messages: empty application messages. message bytes: one application message. over: what refuses one more step. Heap is not measured here.
| shape | rail | axis | widest | frame | SPP CPI | bytes | accounts | signatures | over |
|---|---|---|---|---|---|---|---|---|---|
| 1x2 | confidential signer | messages | 56 | 3146 | 3079 | 4063 | 20 | 3 | 1 B |
| 1x2 | confidential signer | message bytes | 1903 | 3179 | 3112 | 4096 | 20 | 3 | 1 B |
| 1x2 | confidential in-proof | messages | 57 | 3276 | 3210 | 4096 | 19 | 2 | 34 B |
| 1x2 | confidential in-proof | message bytes | 1904 | 3276 | 3210 | 4096 | 19 | 2 | 1 B |
| 1x2 | anonymous hidden | reply | 2001 | 3372 | 3345 | 4096 | 16 | 2 | 1 B |
| 1x2 | anonymous hidden | messages | 55 | 3339 | 3311 | 4063 | 16 | 2 | 1 B |
| 1x2 | anonymous hidden | message bytes | 1869 | 3372 | 3344 | 4096 | 16 | 2 | 1 B |
| 1x2 | anonymous public owners | reply | 1872 | 3179 | 3119 | 4096 | 20 | 3 | 1 B |
| 1x2 | anonymous public owners | messages | 52 | 3173 | 3112 | 4090 | 20 | 3 | 28 B |
| 1x2 | anonymous public owners | message bytes | 1740 | 3179 | 3118 | 4096 | 20 | 3 | 1 B |
| 1x2 | confidential entry | messages | 52 | 3154 | 3087 | 4071 | 20 | 3 | 9 B |
| 1x2 | confidential entry | message bytes | 1759 | 3179 | 3112 | 4096 | 20 | 3 | 1 B |
| 1x2 | anonymous entry | reply | 1825 | 3372 | 3313 | 4096 | 16 | 2 | 1 B |
| 1x2 | anonymous entry | messages | 50 | 3345 | 3285 | 4069 | 16 | 2 | 7 B |
| 1x2 | anonymous entry | message bytes | 1693 | 3372 | 3312 | 4096 | 16 | 2 | 1 B |
| 1x4 | confidential signer | messages | 44 | 3164 | 3165 | 4081 | 20 | 3 | 19 B |
| 1x4 | confidential signer | message bytes | 1477 | 3179 | 3180 | 4096 | 20 | 3 | 1 B |
| 1x4 | confidential in-proof | messages | 44 | 3260 | 3262 | 4080 | 19 | 2 | 18 B |
| 1x4 | confidential in-proof | message bytes | 1478 | 3276 | 3278 | 4096 | 19 | 2 | 1 B |
| 1x4 | anonymous hidden | reply | 525 | 3372 | 3413 | 4096 | 16 | 2 | 3 B |
| 1x4 | anonymous hidden | messages | 37 | 3347 | 3387 | 4071 | 16 | 2 | 9 B |
| 1x4 | anonymous hidden | message bytes | 1249 | 3372 | 3412 | 4096 | 16 | 2 | 1 B |
| 1x4 | anonymous public owners | reply | 482 | 3179 | 3187 | 4096 | 20 | 3 | 3 B |
| 1x4 | anonymous public owners | messages | 33 | 3147 | 3154 | 4064 | 20 | 3 | 2 B |
| 1x4 | anonymous public owners | message bytes | 1120 | 3179 | 3186 | 4096 | 20 | 3 | 1 B |
| 1x4 | confidential entry | messages | 40 | 3172 | 3173 | 4089 | 20 | 3 | 27 B |
| 1x4 | confidential entry | message bytes | 1333 | 3179 | 3180 | 4096 | 20 | 3 | 1 B |
| 1x4 | anonymous entry | reply | 466 | 3371 | 3380 | 4095 | 16 | 2 | 2 B |
| 1x4 | anonymous entry | messages | 32 | 3353 | 3361 | 4077 | 16 | 2 | 15 B |
| 1x4 | anonymous entry | message bytes | 1073 | 3372 | 3380 | 4096 | 16 | 2 | 1 B |
| 1x8 | confidential signer | messages | 19 | 3166 | 3303 | 4083 | 20 | 3 | 21 B |
| 1x8 | confidential signer | message bytes | 625 | 3179 | 3316 | 4096 | 20 | 3 | 1 B |
| 1x8 | confidential in-proof | messages | 19 | 3262 | 3400 | 4082 | 19 | 2 | 20 B |
| 1x8 | confidential in-proof | message bytes | 626 | 3276 | 3414 | 4096 | 19 | 2 | 1 B |
| 1x8 | anonymous hidden | reply | 103 | 3370 | 3547 | 4094 | 16 | 2 | 5 B |
| 1x8 | anonymous hidden | messages | 1 | 3363 | 3539 | 4087 | 16 | 2 | 25 B |
| 1x8 | anonymous hidden | message bytes | 9 | 3372 | 3548 | 4096 | 16 | 2 | 1 B |
| 1x8 | anonymous public owners | reply | none | - | - | - | - | - | 85 B |
| 1x8 | anonymous public owners | messages | none | - | - | - | - | - | 85 B |
| 1x8 | anonymous public owners | message bytes | none | - | - | - | - | - | 120 B |
| 1x8 | confidential entry | messages | 15 | 3174 | 3311 | 4091 | 20 | 3 | 29 B |
| 1x8 | confidential entry | message bytes | 481 | 3179 | 3316 | 4096 | 20 | 3 | 1 B |
| 1x8 | anonymous entry | reply | none | - | - | - | - | - | 132 B |
| 1x8 | anonymous entry | messages | none | - | - | - | - | - | 132 B |
| 1x8 | anonymous entry | message bytes | none | - | - | - | - | - | 167 B |
| 2x2 | confidential signer | messages | 55 | 3144 | 3078 | 4094 | 21 | 3 | 32 B |
| 2x2 | confidential signer | message bytes | 1838 | 3146 | 3080 | 4096 | 21 | 3 | 1 B |
| 2x2 | confidential in-proof | messages | 55 | 3240 | 3175 | 4093 | 20 | 2 | 31 B |
| 2x2 | confidential in-proof | message bytes | 1839 | 3243 | 3178 | 4096 | 20 | 2 | 1 B |
| 2x2 | anonymous hidden | reply | 1936 | 3339 | 3313 | 4096 | 17 | 2 | 1 B |
| 2x2 | anonymous hidden | messages | 54 | 3337 | 3310 | 4094 | 17 | 2 | 32 B |
| 2x2 | anonymous hidden | message bytes | 1804 | 3339 | 3312 | 4096 | 17 | 2 | 1 B |
| 2x2 | anonymous public owners | reply | 1807 | 3146 | 3087 | 4096 | 21 | 3 | 1 B |
| 2x2 | anonymous public owners | messages | 50 | 3137 | 3077 | 4087 | 21 | 3 | 25 B |
| 2x2 | anonymous public owners | message bytes | 1675 | 3146 | 3086 | 4096 | 21 | 3 | 1 B |
| 2x2 | confidential entry | messages | 46 | 3118 | 3052 | 4068 | 21 | 3 | 6 B |
| 2x2 | confidential entry | message bytes | 1558 | 3146 | 3080 | 4096 | 21 | 3 | 1 B |
| 2x2 | anonymous entry | reply | 1624 | 3339 | 3281 | 4096 | 17 | 2 | 1 B |
| 2x2 | anonymous entry | messages | 44 | 3309 | 3250 | 4066 | 17 | 2 | 4 B |
| 2x2 | anonymous entry | message bytes | 1492 | 3339 | 3280 | 4096 | 17 | 2 | 1 B |
| 2x2 | delegate | messages | 62 | 3334 | 3316 | 4088 | 17 | 2 | 26 B |
| 2x2 | delegate | message bytes | 2082 | 3342 | 3324 | 4096 | 17 | 2 | 1 B |
| 2x2 | anonymous delegate | reply | 2051 | 3342 | 3331 | 4096 | 17 | 2 | 1 B |
| 2x2 | anonymous delegate | messages | 57 | 3327 | 3315 | 4081 | 17 | 2 | 19 B |
| 2x2 | anonymous delegate | message bytes | 1919 | 3342 | 3330 | 4096 | 17 | 2 | 1 B |
| 2x4 | confidential signer | messages | 42 | 3128 | 3130 | 4078 | 21 | 3 | 16 B |
| 2x4 | confidential signer | message bytes | 1412 | 3146 | 3148 | 4096 | 21 | 3 | 1 B |
| 2x4 | confidential in-proof | messages | 42 | 3224 | 3227 | 4077 | 20 | 2 | 15 B |
| 2x4 | confidential in-proof | message bytes | 1413 | 3243 | 3246 | 4096 | 20 | 2 | 1 B |
| 2x4 | anonymous hidden | reply | 503 | 3338 | 3380 | 4095 | 17 | 2 | 2 B |
| 2x4 | anonymous hidden | messages | 35 | 3311 | 3352 | 4068 | 17 | 2 | 6 B |
| 2x4 | anonymous hidden | message bytes | 1184 | 3339 | 3380 | 4096 | 17 | 2 | 1 B |
| 2x4 | anonymous public owners | reply | 460 | 3145 | 3154 | 4095 | 21 | 3 | 2 B |
| 2x4 | anonymous public owners | messages | 32 | 3145 | 3153 | 4095 | 21 | 3 | 33 B |
| 2x4 | anonymous public owners | message bytes | 1055 | 3146 | 3154 | 4096 | 21 | 3 | 1 B |
| 2x4 | confidential entry | messages | 34 | 3136 | 3138 | 4086 | 21 | 3 | 24 B |
| 2x4 | confidential entry | message bytes | 1132 | 3146 | 3148 | 4096 | 21 | 3 | 1 B |
| 2x4 | anonymous entry | reply | 399 | 3338 | 3348 | 4095 | 17 | 2 | 2 B |
| 2x4 | anonymous entry | messages | 26 | 3317 | 3326 | 4074 | 17 | 2 | 12 B |
| 2x4 | anonymous entry | message bytes | 872 | 3339 | 3348 | 4096 | 17 | 2 | 1 B |
| 1x16 | confidential signer | messages | none | - | - | - | - | - | 1045 B |
| 1x16 | confidential signer | message bytes | none | - | - | - | - | - | 1080 B |
| 1x16 | confidential in-proof | messages | none | - | - | - | - | - | 1044 B |
| 1x16 | confidential in-proof | message bytes | none | - | - | - | - | - | 1079 B |
| 1x16 | anonymous hidden | reply | none | - | - | - | - | - | 2437 B |
| 1x16 | anonymous hidden | messages | none | - | - | - | - | - | 2437 B |
| 1x16 | anonymous hidden | message bytes | none | - | - | - | - | - | 2472 B |
| 1x16 | anonymous public owners | reply | none | - | - | - | - | - | 2566 B |
| 1x16 | anonymous public owners | messages | none | - | - | - | - | - | 2566 B |
| 1x16 | anonymous public owners | message bytes | none | - | - | - | - | - | 2601 B |
| 2x8 | confidential signer | messages | 17 | 3130 | 3268 | 4080 | 21 | 3 | 18 B |
| 2x8 | confidential signer | message bytes | 560 | 3146 | 3284 | 4096 | 21 | 3 | 1 B |
| 2x8 | confidential in-proof | messages | 17 | 3226 | 3365 | 4079 | 20 | 2 | 17 B |
| 2x8 | confidential in-proof | message bytes | 561 | 3243 | 3382 | 4096 | 20 | 2 | 1 B |
| 2x8 | anonymous hidden | reply | none | - | - | - | - | - | 21 B |
| 2x8 | anonymous hidden | messages | none | - | - | - | - | - | 21 B |
| 2x8 | anonymous hidden | message bytes | none | - | - | - | - | - | 56 B |
| 2x8 | anonymous public owners | reply | none | - | - | - | - | - | 150 B |
| 2x8 | anonymous public owners | messages | none | - | - | - | - | - | 150 B |
| 2x8 | anonymous public owners | message bytes | none | - | - | - | - | - | 185 B |
| 2x8 | confidential entry | messages | 9 | 3138 | 3276 | 4088 | 21 | 3 | 26 B |
| 2x8 | confidential entry | message bytes | 280 | 3146 | 3284 | 4096 | 21 | 3 | 1 B |
| 2x8 | anonymous entry | reply | none | - | - | - | - | - | 333 B |
| 2x8 | anonymous entry | messages | none | - | - | - | - | - | 333 B |
| 2x8 | anonymous entry | message bytes | none | - | - | - | - | - | 368 B |
| 3x2 | confidential signer | messages | 53 | 3108 | 3043 | 4091 | 22 | 3 | 29 B |
| 3x2 | confidential signer | message bytes | 1773 | 3113 | 3048 | 4096 | 22 | 3 | 1 B |
| 3x2 | confidential in-proof | messages | 53 | 3204 | 3140 | 4090 | 21 | 2 | 28 B |
| 3x2 | confidential in-proof | message bytes | 1774 | 3210 | 3146 | 4096 | 21 | 2 | 1 B |
| 3x2 | anonymous hidden | reply | 1871 | 3306 | 3281 | 4096 | 18 | 2 | 1 B |
| 3x2 | anonymous hidden | messages | 52 | 3301 | 3275 | 4091 | 18 | 2 | 29 B |
| 3x2 | anonymous hidden | message bytes | 1739 | 3306 | 3280 | 4096 | 18 | 2 | 1 B |
| 3x2 | anonymous public owners | reply | 1742 | 3113 | 3055 | 4096 | 22 | 3 | 1 B |
| 3x2 | anonymous public owners | messages | 48 | 3101 | 3042 | 4084 | 22 | 3 | 22 B |
| 3x2 | anonymous public owners | message bytes | 1610 | 3113 | 3054 | 4096 | 22 | 3 | 1 B |
| 3x2 | confidential entry | messages | 40 | 3082 | 3017 | 4065 | 22 | 3 | 3 B |
| 3x2 | confidential entry | message bytes | 1357 | 3113 | 3048 | 4096 | 22 | 3 | 1 B |
| 3x2 | anonymous entry | reply | 1423 | 3306 | 3249 | 4096 | 18 | 2 | 1 B |
| 3x2 | anonymous entry | messages | 38 | 3273 | 3215 | 4063 | 18 | 2 | 1 B |
| 3x2 | anonymous entry | message bytes | 1291 | 3306 | 3248 | 4096 | 18 | 2 | 1 B |
| 3x4 | confidential signer | messages | 40 | 3092 | 3095 | 4075 | 22 | 3 | 13 B |
| 3x4 | confidential signer | message bytes | 1347 | 3113 | 3116 | 4096 | 22 | 3 | 1 B |
| 3x4 | confidential in-proof | messages | 40 | 3188 | 3192 | 4074 | 21 | 2 | 12 B |
| 3x4 | confidential in-proof | message bytes | 1348 | 3210 | 3214 | 4096 | 21 | 2 | 1 B |
| 3x4 | anonymous hidden | reply | 481 | 3304 | 3347 | 4094 | 18 | 2 | 1 B |
| 3x4 | anonymous hidden | messages | 33 | 3275 | 3317 | 4065 | 18 | 2 | 3 B |
| 3x4 | anonymous hidden | message bytes | 1119 | 3306 | 3348 | 4096 | 18 | 2 | 1 B |
| 3x4 | anonymous public owners | reply | 438 | 3111 | 3121 | 4094 | 22 | 3 | 1 B |
| 3x4 | anonymous public owners | messages | 30 | 3109 | 3118 | 4092 | 22 | 3 | 30 B |
| 3x4 | anonymous public owners | message bytes | 990 | 3113 | 3122 | 4096 | 22 | 3 | 1 B |
| 3x4 | confidential entry | messages | 28 | 3100 | 3103 | 4083 | 22 | 3 | 21 B |
| 3x4 | confidential entry | message bytes | 931 | 3113 | 3116 | 4096 | 22 | 3 | 1 B |
| 3x4 | anonymous entry | reply | 332 | 3305 | 3316 | 4095 | 18 | 2 | 2 B |
| 3x4 | anonymous entry | messages | 20 | 3281 | 3291 | 4071 | 18 | 2 | 9 B |
| 3x4 | anonymous entry | message bytes | 671 | 3306 | 3316 | 4096 | 18 | 2 | 1 B |
| 2x16 | confidential signer | messages | none | - | - | - | - | - | 1110 B |
| 2x16 | confidential signer | message bytes | none | - | - | - | - | - | 1145 B |
| 2x16 | confidential in-proof | messages | none | - | - | - | - | - | 1109 B |
| 2x16 | confidential in-proof | message bytes | none | - | - | - | - | - | 1144 B |
| 2x16 | anonymous hidden | reply | none | - | - | - | - | - | 2502 B |
| 2x16 | anonymous hidden | messages | none | - | - | - | - | - | 2502 B |
| 2x16 | anonymous hidden | message bytes | none | - | - | - | - | - | 2537 B |
| 2x16 | anonymous public owners | reply | none | - | - | - | - | - | 2631 B |
| 2x16 | anonymous public owners | messages | none | - | - | - | - | - | 2631 B |
| 2x16 | anonymous public owners | message bytes | none | - | - | - | - | - | 2666 B |
| 3x8 | confidential signer | messages | 15 | 3094 | 3233 | 4077 | 22 | 3 | 15 B |
| 3x8 | confidential signer | message bytes | 495 | 3113 | 3252 | 4096 | 22 | 3 | 1 B |
| 3x8 | confidential in-proof | messages | 15 | 3190 | 3330 | 4076 | 21 | 2 | 14 B |
| 3x8 | confidential in-proof | message bytes | 496 | 3210 | 3350 | 4096 | 21 | 2 | 1 B |
| 3x8 | anonymous hidden | reply | none | - | - | - | - | - | 86 B |
| 3x8 | anonymous hidden | messages | none | - | - | - | - | - | 86 B |
| 3x8 | anonymous hidden | message bytes | none | - | - | - | - | - | 121 B |
| 3x8 | anonymous public owners | reply | none | - | - | - | - | - | 215 B |
| 3x8 | anonymous public owners | messages | none | - | - | - | - | - | 215 B |
| 3x8 | anonymous public owners | message bytes | none | - | - | - | - | - | 250 B |
| 3x8 | confidential entry | messages | 3 | 3102 | 3241 | 4085 | 22 | 3 | 23 B |
| 3x8 | confidential entry | message bytes | 79 | 3113 | 3252 | 4096 | 22 | 3 | 1 B |
| 3x8 | anonymous entry | reply | none | - | - | - | - | - | 534 B |
| 3x8 | anonymous entry | messages | none | - | - | - | - | - | 534 B |
| 3x8 | anonymous entry | message bytes | none | - | - | - | - | - | 569 B |
| 4x2 | confidential signer | messages | 51 | 3072 | 3008 | 4088 | 23 | 3 | 26 B |
| 4x2 | confidential signer | message bytes | 1708 | 3080 | 3016 | 4096 | 23 | 3 | 1 B |
| 4x2 | confidential in-proof | messages | 51 | 3168 | 3105 | 4087 | 22 | 2 | 25 B |
| 4x2 | confidential in-proof | message bytes | 1709 | 3177 | 3114 | 4096 | 22 | 2 | 1 B |
| 4x2 | anonymous hidden | reply | 1806 | 3273 | 3249 | 4096 | 19 | 2 | 1 B |
| 4x2 | anonymous hidden | messages | 50 | 3265 | 3240 | 4088 | 19 | 2 | 26 B |
| 4x2 | anonymous hidden | message bytes | 1674 | 3273 | 3248 | 4096 | 19 | 2 | 1 B |
| 4x2 | anonymous public owners | reply | 1677 | 3080 | 3023 | 4096 | 23 | 3 | 1 B |
| 4x2 | anonymous public owners | messages | 46 | 3065 | 3007 | 4081 | 23 | 3 | 19 B |
| 4x2 | anonymous public owners | message bytes | 1545 | 3080 | 3022 | 4096 | 23 | 3 | 1 B |
| 4x2 | confidential entry | messages | 35 | 3080 | 3016 | 4096 | 23 | 3 | 34 B |
| 4x2 | confidential entry | message bytes | 1156 | 3080 | 3016 | 4096 | 23 | 3 | 1 B |
| 4x2 | anonymous entry | reply | 1222 | 3273 | 3217 | 4096 | 19 | 2 | 1 B |
| 4x2 | anonymous entry | messages | 33 | 3271 | 3214 | 4094 | 19 | 2 | 32 B |
| 4x2 | anonymous entry | message bytes | 1090 | 3273 | 3216 | 4096 | 19 | 2 | 1 B |
| 4x4 | confidential signer | messages | 38 | 3056 | 3060 | 4072 | 23 | 3 | 10 B |
| 4x4 | confidential signer | message bytes | 1282 | 3080 | 3084 | 4096 | 23 | 3 | 1 B |
| 4x4 | confidential in-proof | messages | 38 | 3152 | 3157 | 4071 | 22 | 2 | 9 B |
| 4x4 | confidential in-proof | message bytes | 1283 | 3177 | 3182 | 4096 | 22 | 2 | 1 B |
| 4x4 | anonymous hidden | reply | 460 | 3273 | 3317 | 4096 | 19 | 2 | 3 B |
| 4x4 | anonymous hidden | messages | 32 | 3273 | 3316 | 4096 | 19 | 2 | 34 B |
| 4x4 | anonymous hidden | message bytes | 1054 | 3273 | 3316 | 4096 | 19 | 2 | 1 B |
| 4x4 | anonymous public owners | reply | 417 | 3080 | 3091 | 4096 | 23 | 3 | 3 B |
| 4x4 | anonymous public owners | messages | 28 | 3073 | 3083 | 4089 | 23 | 3 | 27 B |
| 4x4 | anonymous public owners | message bytes | 925 | 3080 | 3090 | 4096 | 23 | 3 | 1 B |
| 4x4 | confidential entry | messages | 22 | 3064 | 3068 | 4080 | 23 | 3 | 18 B |
| 4x4 | confidential entry | message bytes | 730 | 3080 | 3084 | 4096 | 23 | 3 | 1 B |
| 4x4 | anonymous entry | reply | 265 | 3272 | 3284 | 4095 | 19 | 2 | 2 B |
| 4x4 | anonymous entry | messages | 14 | 3245 | 3256 | 4068 | 19 | 2 | 6 B |
| 4x4 | anonymous entry | message bytes | 470 | 3273 | 3284 | 4096 | 19 | 2 | 1 B |
| 4x4 | delegate | messages | 45 | 3246 | 3298 | 4066 | 19 | 2 | 4 B |
| 4x4 | delegate | message bytes | 1526 | 3276 | 3328 | 4096 | 19 | 2 | 1 B |
| 4x4 | anonymous delegate | reply | 498 | 3275 | 3334 | 4095 | 19 | 2 | 2 B |
| 4x4 | anonymous delegate | messages | 35 | 3263 | 3321 | 4083 | 19 | 2 | 21 B |
| 4x4 | anonymous delegate | message bytes | 1169 | 3276 | 3334 | 4096 | 19 | 2 | 1 B |
| 4x8 | confidential signer | messages | 13 | 3058 | 3198 | 4074 | 23 | 3 | 12 B |
| 4x8 | confidential signer | message bytes | 430 | 3080 | 3220 | 4096 | 23 | 3 | 1 B |
| 4x8 | confidential in-proof | messages | 13 | 3154 | 3295 | 4073 | 22 | 2 | 11 B |
| 4x8 | confidential in-proof | message bytes | 431 | 3177 | 3318 | 4096 | 22 | 2 | 1 B |
| 4x8 | anonymous hidden | reply | none | - | - | - | - | - | 151 B |
| 4x8 | anonymous hidden | messages | none | - | - | - | - | - | 151 B |
| 4x8 | anonymous hidden | message bytes | none | - | - | - | - | - | 186 B |
| 4x8 | anonymous public owners | reply | none | - | - | - | - | - | 280 B |
| 4x8 | anonymous public owners | messages | none | - | - | - | - | - | 280 B |
| 4x8 | anonymous public owners | message bytes | none | - | - | - | - | - | 315 B |
| 4x8 | confidential entry | messages | none | - | - | - | - | - | 87 B |
| 4x8 | confidential entry | message bytes | none | - | - | - | - | - | 122 B |
| 4x8 | anonymous entry | reply | none | - | - | - | - | - | 735 B |
| 4x8 | anonymous entry | messages | none | - | - | - | - | - | 735 B |
| 4x8 | anonymous entry | message bytes | none | - | - | - | - | - | 770 B |
| 5x2 | confidential signer | messages | 49 | 3036 | 2973 | 4085 | 24 | 3 | 23 B |
| 5x2 | confidential signer | message bytes | 1643 | 3047 | 2984 | 4096 | 24 | 3 | 1 B |
| 5x2 | confidential in-proof | messages | 49 | 3132 | 3070 | 4084 | 23 | 2 | 22 B |
| 5x2 | confidential in-proof | message bytes | 1644 | 3144 | 3082 | 4096 | 23 | 2 | 1 B |
| 5x2 | anonymous hidden | reply | 1741 | 3240 | 3217 | 4096 | 20 | 2 | 1 B |
| 5x2 | anonymous hidden | messages | 48 | 3229 | 3205 | 4085 | 20 | 2 | 23 B |
| 5x2 | anonymous hidden | message bytes | 1609 | 3240 | 3216 | 4096 | 20 | 2 | 1 B |
| 5x2 | anonymous public owners | reply | 1612 | 3047 | 2991 | 4096 | 24 | 3 | 1 B |
| 5x2 | anonymous public owners | messages | 44 | 3029 | 2972 | 4078 | 24 | 3 | 16 B |
| 5x2 | anonymous public owners | message bytes | 1480 | 3047 | 2990 | 4096 | 24 | 3 | 1 B |
| 5x2 | confidential entry | messages | 29 | 3044 | 2981 | 4093 | 24 | 3 | 31 B |
| 5x2 | confidential entry | message bytes | 955 | 3047 | 2984 | 4096 | 24 | 3 | 1 B |
| 5x2 | anonymous entry | reply | 1021 | 3240 | 3185 | 4096 | 20 | 2 | 1 B |
| 5x2 | anonymous entry | messages | 27 | 3235 | 3179 | 4091 | 20 | 2 | 29 B |
| 5x2 | anonymous entry | message bytes | 889 | 3240 | 3184 | 4096 | 20 | 2 | 1 B |
| 5x4 | confidential signer | messages | 36 | 3020 | 3025 | 4069 | 24 | 3 | 7 B |
| 5x4 | confidential signer | message bytes | 1217 | 3047 | 3052 | 4096 | 24 | 3 | 1 B |
| 5x4 | confidential in-proof | messages | 36 | 3116 | 3122 | 4068 | 23 | 2 | 6 B |
| 5x4 | confidential in-proof | message bytes | 1218 | 3144 | 3150 | 4096 | 23 | 2 | 1 B |
| 5x4 | anonymous hidden | reply | 438 | 3239 | 3284 | 4095 | 20 | 2 | 2 B |
| 5x4 | anonymous hidden | messages | 30 | 3237 | 3281 | 4093 | 20 | 2 | 31 B |
| 5x4 | anonymous hidden | message bytes | 989 | 3240 | 3284 | 4096 | 20 | 2 | 1 B |
| 5x4 | anonymous public owners | reply | 395 | 3046 | 3058 | 4095 | 24 | 3 | 2 B |
| 5x4 | anonymous public owners | messages | 26 | 3037 | 3048 | 4086 | 24 | 3 | 24 B |
| 5x4 | anonymous public owners | message bytes | 860 | 3047 | 3058 | 4096 | 24 | 3 | 1 B |
| 5x4 | confidential entry | messages | 16 | 3028 | 3033 | 4077 | 24 | 3 | 15 B |
| 5x4 | confidential entry | message bytes | 529 | 3047 | 3052 | 4096 | 24 | 3 | 1 B |
| 5x4 | anonymous entry | reply | 198 | 3239 | 3252 | 4095 | 20 | 2 | 2 B |
| 5x4 | anonymous entry | messages | 8 | 3209 | 3221 | 4065 | 20 | 2 | 3 B |
| 5x4 | anonymous entry | message bytes | 269 | 3240 | 3252 | 4096 | 20 | 2 | 1 B |
| 4x16 | confidential signer | messages | none | - | - | - | - | - | 1240 B |
| 4x16 | confidential signer | message bytes | none | - | - | - | - | - | 1275 B |
| 4x16 | confidential in-proof | messages | none | - | - | - | - | - | 1239 B |
| 4x16 | confidential in-proof | message bytes | none | - | - | - | - | - | 1274 B |
| 4x16 | anonymous hidden | reply | none | - | - | - | - | - | 2632 B |
| 4x16 | anonymous hidden | messages | none | - | - | - | - | - | 2632 B |
| 4x16 | anonymous hidden | message bytes | none | - | - | - | - | - | 2667 B |
| 4x16 | anonymous public owners | reply | none | - | - | - | - | - | 2761 B |
| 4x16 | anonymous public owners | messages | none | - | - | - | - | - | 2761 B |
| 4x16 | anonymous public owners | message bytes | none | - | - | - | - | - | 2796 B |
| 5x8 | confidential signer | messages | 11 | 3022 | 3163 | 4071 | 24 | 3 | 9 B |
| 5x8 | confidential signer | message bytes | 365 | 3047 | 3188 | 4096 | 24 | 3 | 1 B |
| 5x8 | confidential in-proof | messages | 11 | 3118 | 3260 | 4070 | 23 | 2 | 8 B |
| 5x8 | confidential in-proof | message bytes | 366 | 3144 | 3286 | 4096 | 23 | 2 | 1 B |
| 5x8 | anonymous hidden | reply | none | - | - | - | - | - | 216 B |
| 5x8 | anonymous hidden | messages | none | - | - | - | - | - | 216 B |
| 5x8 | anonymous hidden | message bytes | none | - | - | - | - | - | 251 B |
| 5x8 | anonymous public owners | reply | none | - | - | - | - | - | 345 B |
| 5x8 | anonymous public owners | messages | none | - | - | - | - | - | 345 B |
| 5x8 | anonymous public owners | message bytes | none | - | - | - | - | - | 380 B |
| 5x8 | confidential entry | messages | none | - | - | - | - | - | 288 B |
| 5x8 | confidential entry | message bytes | none | - | - | - | - | - | 323 B |
| 5x8 | anonymous entry | reply | none | - | - | - | - | - | 936 B |
| 5x8 | anonymous entry | messages | none | - | - | - | - | - | 936 B |
| 5x8 | anonymous entry | message bytes | none | - | - | - | - | - | 971 B |
| 6x2 | confidential signer | messages | 47 | 3000 | 2938 | 4082 | 25 | 3 | 20 B |
| 6x2 | confidential signer | message bytes | 1578 | 3014 | 2952 | 4096 | 25 | 3 | 1 B |
| 6x2 | confidential in-proof | messages | 47 | 3096 | 3035 | 4081 | 24 | 2 | 19 B |
| 6x2 | confidential in-proof | message bytes | 1579 | 3111 | 3050 | 4096 | 24 | 2 | 1 B |
| 6x2 | anonymous hidden | reply | 1676 | 3207 | 3185 | 4096 | 21 | 2 | 1 B |
| 6x2 | anonymous hidden | messages | 46 | 3193 | 3170 | 4082 | 21 | 2 | 20 B |
| 6x2 | anonymous hidden | message bytes | 1544 | 3207 | 3184 | 4096 | 21 | 2 | 1 B |
| 6x2 | anonymous public owners | reply | 1547 | 3014 | 2959 | 4096 | 25 | 3 | 1 B |
| 6x2 | anonymous public owners | messages | 42 | 2993 | 2937 | 4075 | 25 | 3 | 13 B |
| 6x2 | anonymous public owners | message bytes | 1415 | 3014 | 2958 | 4096 | 25 | 3 | 1 B |
| 6x2 | confidential entry | messages | 23 | 3008 | 2946 | 4090 | 25 | 3 | 28 B |
| 6x2 | confidential entry | message bytes | 754 | 3014 | 2952 | 4096 | 25 | 3 | 1 B |
| 6x2 | anonymous entry | reply | 820 | 3207 | 3153 | 4096 | 21 | 2 | 1 B |
| 6x2 | anonymous entry | messages | 21 | 3199 | 3144 | 4088 | 21 | 2 | 26 B |
| 6x2 | anonymous entry | message bytes | 688 | 3207 | 3152 | 4096 | 21 | 2 | 1 B |
| 6x4 | confidential signer | messages | 34 | 2984 | 2990 | 4066 | 25 | 3 | 4 B |
| 6x4 | confidential signer | message bytes | 1152 | 3014 | 3020 | 4096 | 25 | 3 | 1 B |
| 6x4 | confidential in-proof | messages | 34 | 3080 | 3087 | 4065 | 24 | 2 | 3 B |
| 6x4 | confidential in-proof | message bytes | 1153 | 3111 | 3118 | 4096 | 24 | 2 | 1 B |
| 6x4 | anonymous hidden | reply | 416 | 3205 | 3251 | 4094 | 21 | 2 | 1 B |
| 6x4 | anonymous hidden | messages | 28 | 3201 | 3246 | 4090 | 21 | 2 | 28 B |
| 6x4 | anonymous hidden | message bytes | 924 | 3207 | 3252 | 4096 | 21 | 2 | 1 B |
| 6x4 | anonymous public owners | reply | 373 | 3012 | 3025 | 4094 | 25 | 3 | 1 B |
| 6x4 | anonymous public owners | messages | 24 | 3001 | 3013 | 4083 | 25 | 3 | 21 B |
| 6x4 | anonymous public owners | message bytes | 795 | 3014 | 3026 | 4096 | 25 | 3 | 1 B |
| 6x4 | confidential entry | messages | 10 | 2992 | 2998 | 4074 | 25 | 3 | 12 B |
| 6x4 | confidential entry | message bytes | 328 | 3014 | 3020 | 4096 | 25 | 3 | 1 B |
| 6x4 | anonymous entry | reply | 131 | 3206 | 3220 | 4095 | 21 | 2 | 2 B |
| 6x4 | anonymous entry | messages | 3 | 3207 | 3220 | 4096 | 21 | 2 | 34 B |
| 6x4 | anonymous entry | message bytes | 68 | 3207 | 3220 | 4096 | 21 | 2 | 1 B |
| 6x8 | confidential signer | messages | 9 | 2986 | 3128 | 4068 | 25 | 3 | 6 B |
| 6x8 | confidential signer | message bytes | 300 | 3014 | 3156 | 4096 | 25 | 3 | 1 B |
| 6x8 | confidential in-proof | messages | 9 | 3082 | 3225 | 4067 | 24 | 2 | 5 B |
| 6x8 | confidential in-proof | message bytes | 301 | 3111 | 3254 | 4096 | 24 | 2 | 1 B |
| 6x8 | anonymous hidden | reply | none | - | - | - | - | - | 281 B |
| 6x8 | anonymous hidden | messages | none | - | - | - | - | - | 281 B |
| 6x8 | anonymous hidden | message bytes | none | - | - | - | - | - | 316 B |
| 6x8 | anonymous public owners | reply | none | - | - | - | - | - | 410 B |
| 6x8 | anonymous public owners | messages | none | - | - | - | - | - | 410 B |
| 6x8 | anonymous public owners | message bytes | none | - | - | - | - | - | 445 B |
| 6x8 | confidential entry | messages | none | - | - | - | - | - | 489 B |
| 6x8 | confidential entry | message bytes | none | - | - | - | - | - | 524 B |
| 6x8 | anonymous entry | reply | none | - | - | - | - | - | 1137 B |
| 6x8 | anonymous entry | messages | none | - | - | - | - | - | 1137 B |
| 6x8 | anonymous entry | message bytes | none | - | - | - | - | - | 1172 B |
| 8x2 | confidential signer | messages | 43 | 2928 | 2868 | 4076 | 27 | 3 | 14 B |
| 8x2 | confidential signer | message bytes | 1448 | 2948 | 2888 | 4096 | 27 | 3 | 1 B |
| 8x2 | confidential in-proof | messages | 43 | 3024 | 2965 | 4075 | 26 | 2 | 13 B |
| 8x2 | confidential in-proof | message bytes | 1449 | 3045 | 2986 | 4096 | 26 | 2 | 1 B |
| 8x2 | anonymous hidden | reply | 1546 | 3141 | 3121 | 4096 | 23 | 2 | 1 B |
| 8x2 | anonymous hidden | messages | 42 | 3121 | 3100 | 4076 | 23 | 2 | 14 B |
| 8x2 | anonymous hidden | message bytes | 1414 | 3141 | 3120 | 4096 | 23 | 2 | 1 B |
| 8x2 | anonymous public owners | reply | 1417 | 2948 | 2895 | 4096 | 27 | 3 | 1 B |
| 8x2 | anonymous public owners | messages | 38 | 2921 | 2867 | 4069 | 27 | 3 | 7 B |
| 8x2 | anonymous public owners | message bytes | 1285 | 2948 | 2894 | 4096 | 27 | 3 | 1 B |
| 8x2 | confidential entry | messages | 11 | 2936 | 2876 | 4084 | 27 | 3 | 22 B |
| 8x2 | confidential entry | message bytes | 352 | 2948 | 2888 | 4096 | 27 | 3 | 1 B |
| 8x2 | anonymous entry | reply | 418 | 3141 | 3089 | 4096 | 23 | 2 | 1 B |
| 8x2 | anonymous entry | messages | 9 | 3127 | 3074 | 4082 | 23 | 2 | 20 B |
| 8x2 | anonymous entry | message bytes | 286 | 3141 | 3088 | 4096 | 23 | 2 | 1 B |
| 8x4 | confidential signer | messages | 31 | 2946 | 2954 | 4094 | 27 | 3 | 32 B |
| 8x4 | confidential signer | message bytes | 1022 | 2948 | 2956 | 4096 | 27 | 3 | 1 B |
| 8x4 | confidential in-proof | messages | 31 | 3042 | 3051 | 4093 | 26 | 2 | 31 B |
| 8x4 | confidential in-proof | message bytes | 1023 | 3045 | 3054 | 4096 | 26 | 2 | 1 B |
| 8x4 | anonymous hidden | reply | 373 | 3140 | 3188 | 4095 | 23 | 2 | 2 B |
| 8x4 | anonymous hidden | messages | 24 | 3129 | 3176 | 4084 | 23 | 2 | 22 B |
| 8x4 | anonymous hidden | message bytes | 794 | 3141 | 3188 | 4096 | 23 | 2 | 1 B |
| 8x4 | anonymous public owners | reply | 330 | 2947 | 2962 | 4095 | 27 | 3 | 2 B |
| 8x4 | anonymous public owners | messages | 20 | 2929 | 2943 | 4077 | 27 | 3 | 15 B |
| 8x4 | anonymous public owners | message bytes | 665 | 2948 | 2962 | 4096 | 27 | 3 | 1 B |
| 8x4 | confidential entry | messages | none | - | - | - | - | - | 39 B |
| 8x4 | confidential entry | message bytes | none | - | - | - | - | - | 74 B |
| 8x4 | anonymous entry | reply | none | - | - | - | - | - | 299 B |
| 8x4 | anonymous entry | messages | none | - | - | - | - | - | 299 B |
| 8x4 | anonymous entry | message bytes | none | - | - | - | - | - | 334 B |
| 8x8 | confidential signer | messages | 6 | 2948 | 3092 | 4096 | 27 | 3 | 34 B |
| 8x8 | confidential signer | message bytes | 170 | 2948 | 3092 | 4096 | 27 | 3 | 1 B |
| 8x8 | confidential in-proof | messages | 6 | 3044 | 3189 | 4095 | 26 | 2 | 33 B |
| 8x8 | confidential in-proof | message bytes | 171 | 3045 | 3190 | 4096 | 26 | 2 | 1 B |
| 8x8 | anonymous hidden | reply | none | - | - | - | - | - | 411 B |
| 8x8 | anonymous hidden | messages | none | - | - | - | - | - | 411 B |
| 8x8 | anonymous hidden | message bytes | none | - | - | - | - | - | 446 B |
| 8x8 | anonymous public owners | reply | none | - | - | - | - | - | 540 B |
| 8x8 | anonymous public owners | messages | none | - | - | - | - | - | 540 B |
| 8x8 | anonymous public owners | message bytes | none | - | - | - | - | - | 575 B |
| 8x8 | confidential entry | messages | none | - | - | - | - | - | 891 B |
| 8x8 | confidential entry | message bytes | none | - | - | - | - | - | 926 B |
| 8x8 | anonymous entry | reply | none | - | - | - | - | - | 1539 B |
| 8x8 | anonymous entry | messages | none | - | - | - | - | - | 1539 B |
| 8x8 | anonymous entry | message bytes | none | - | - | - | - | - | 1574 B |
| 12x2 | confidential signer | messages | 35 | 2785 | 2728 | 4065 | 31 | 3 | 3 B |
| 12x2 | confidential signer | message bytes | 1187 | 2816 | 2759 | 4096 | 31 | 3 | 1 B |
| 12x2 | confidential in-proof | messages | 35 | 2881 | 2825 | 4064 | 30 | 2 | 2 B |
| 12x2 | confidential in-proof | message bytes | 1188 | 2913 | 2857 | 4096 | 30 | 2 | 1 B |
| 12x2 | anonymous hidden | reply | 1285 | 3009 | 2992 | 4096 | 27 | 2 | 1 B |
| 12x2 | anonymous hidden | messages | 34 | 2978 | 2960 | 4065 | 27 | 2 | 3 B |
| 12x2 | anonymous hidden | message bytes | 1153 | 3009 | 2991 | 4096 | 27 | 2 | 1 B |
| 12x2 | anonymous public owners | reply | 1156 | 2816 | 2766 | 4096 | 31 | 3 | 1 B |
| 12x2 | anonymous public owners | messages | 31 | 2812 | 2761 | 4092 | 31 | 3 | 30 B |
| 12x2 | anonymous public owners | message bytes | 1024 | 2816 | 2765 | 4096 | 31 | 3 | 1 B |
| 12x4 | confidential signer | messages | 23 | 2803 | 2814 | 4083 | 31 | 3 | 21 B |
| 12x4 | confidential signer | message bytes | 761 | 2816 | 2827 | 4096 | 31 | 3 | 1 B |
| 12x4 | confidential in-proof | messages | 23 | 2899 | 2911 | 4082 | 30 | 2 | 20 B |
| 12x4 | confidential in-proof | message bytes | 762 | 2913 | 2925 | 4096 | 30 | 2 | 1 B |
| 12x4 | anonymous hidden | reply | 286 | 3008 | 3059 | 4095 | 27 | 2 | 2 B |
| 12x4 | anonymous hidden | messages | 16 | 2986 | 3036 | 4073 | 27 | 2 | 11 B |
| 12x4 | anonymous hidden | message bytes | 533 | 3009 | 3059 | 4096 | 27 | 2 | 1 B |
| 12x4 | anonymous public owners | reply | 243 | 2815 | 2833 | 4095 | 31 | 3 | 2 B |
| 12x4 | anonymous public owners | messages | 12 | 2786 | 2803 | 4066 | 31 | 3 | 4 B |
| 12x4 | anonymous public owners | message bytes | 404 | 2816 | 2833 | 4096 | 31 | 3 | 1 B |
| 12x8 | confidential signer | messages | none | - | - | - | - | - | 56 B |
| 12x8 | confidential signer | message bytes | none | - | - | - | - | - | 91 B |
| 12x8 | confidential in-proof | messages | none | - | - | - | - | - | 55 B |
| 12x8 | confidential in-proof | message bytes | none | - | - | - | - | - | 90 B |
| 12x8 | anonymous hidden | reply | none | - | - | - | - | - | 672 B |
| 12x8 | anonymous hidden | messages | none | - | - | - | - | - | 672 B |
| 12x8 | anonymous hidden | message bytes | none | - | - | - | - | - | 707 B |
| 12x8 | anonymous public owners | reply | none | - | - | - | - | - | 801 B |
| 12x8 | anonymous public owners | messages | none | - | - | - | - | - | 801 B |
| 12x8 | anonymous public owners | message bytes | none | - | - | - | - | - | 836 B |
| 16x2 | confidential signer | messages | 28 | 2675 | 2622 | 4087 | 35 | 3 | 25 B |
| 16x2 | confidential signer | message bytes | 927 | 2684 | 2631 | 4096 | 35 | 3 | 1 B |
| 16x2 | confidential in-proof | messages | 28 | 2771 | 2719 | 4086 | 34 | 2 | 24 B |
| 16x2 | confidential in-proof | message bytes | 928 | 2781 | 2729 | 4096 | 34 | 2 | 1 B |
| 16x2 | anonymous hidden | reply | 1025 | 2877 | 2864 | 4096 | 31 | 2 | 1 B |
| 16x2 | anonymous hidden | messages | 27 | 2868 | 2854 | 4087 | 31 | 2 | 25 B |
| 16x2 | anonymous hidden | message bytes | 893 | 2877 | 2863 | 4096 | 31 | 2 | 1 B |
| 16x2 | anonymous public owners | reply | 896 | 2684 | 2638 | 4096 | 35 | 3 | 1 B |
| 16x2 | anonymous public owners | messages | 23 | 2668 | 2621 | 4080 | 35 | 3 | 18 B |
| 16x2 | anonymous public owners | message bytes | 764 | 2684 | 2637 | 4096 | 35 | 3 | 1 B |
| 16x4 | confidential signer | messages | 15 | 2659 | 2674 | 4071 | 35 | 3 | 9 B |
| 16x4 | confidential signer | message bytes | 501 | 2684 | 2699 | 4096 | 35 | 3 | 1 B |
| 16x4 | confidential in-proof | messages | 15 | 2755 | 2771 | 4070 | 34 | 2 | 8 B |
| 16x4 | confidential in-proof | message bytes | 502 | 2781 | 2797 | 4096 | 34 | 2 | 1 B |
| 16x4 | anonymous hidden | reply | 199 | 2875 | 2930 | 4094 | 31 | 2 | 1 B |
| 16x4 | anonymous hidden | messages | 9 | 2876 | 2930 | 4095 | 31 | 2 | 33 B |
| 16x4 | anonymous hidden | message bytes | 273 | 2877 | 2931 | 4096 | 31 | 2 | 1 B |
| 16x4 | anonymous public owners | reply | 156 | 2682 | 2704 | 4094 | 35 | 3 | 1 B |
| 16x4 | anonymous public owners | messages | 5 | 2676 | 2697 | 4088 | 35 | 3 | 26 B |
| 16x4 | anonymous public owners | message bytes | 144 | 2684 | 2705 | 4096 | 35 | 3 | 1 B |
| 16x8 | confidential signer | messages | none | - | - | - | - | - | 316 B |
| 16x8 | confidential signer | message bytes | none | - | - | - | - | - | 351 B |
| 16x8 | confidential in-proof | messages | none | - | - | - | - | - | 315 B |
| 16x8 | confidential in-proof | message bytes | none | - | - | - | - | - | 350 B |
| 16x8 | anonymous hidden | reply | none | - | - | - | - | - | 932 B |
| 16x8 | anonymous hidden | messages | none | - | - | - | - | - | 932 B |
| 16x8 | anonymous hidden | message bytes | none | - | - | - | - | - | 967 B |
| 16x8 | anonymous public owners | reply | none | - | - | - | - | - | 1061 B |
| 16x8 | anonymous public owners | messages | none | - | - | - | - | - | 1061 B |
| 16x8 | anonymous public owners | message bytes | none | - | - | - | - | - | 1096 B |
| 24x2 | confidential signer | messages | 12 | 2388 | 2342 | 4064 | 43 | 3 | 2 B |
| 24x2 | confidential signer | message bytes | 406 | 2420 | 2374 | 4096 | 43 | 3 | 1 B |
| 24x2 | confidential in-proof | messages | 12 | 2484 | 2439 | 4063 | 42 | 2 | 1 B |
| 24x2 | confidential in-proof | message bytes | 407 | 2517 | 2472 | 4096 | 42 | 2 | 1 B |
| 24x2 | anonymous hidden | reply | 504 | 2613 | 2607 | 4096 | 39 | 2 | 1 B |
| 24x2 | anonymous hidden | messages | 11 | 2581 | 2574 | 4064 | 39 | 2 | 2 B |
| 24x2 | anonymous hidden | message bytes | 372 | 2613 | 2606 | 4096 | 39 | 2 | 1 B |
| 24x2 | anonymous public owners | reply | 375 | 2420 | 2381 | 4096 | 43 | 3 | 1 B |
| 24x2 | anonymous public owners | messages | 8 | 2415 | 2375 | 4091 | 43 | 3 | 29 B |
| 24x2 | anonymous public owners | message bytes | 243 | 2420 | 2380 | 4096 | 43 | 3 | 1 B |
| 24x4 | confidential signer | messages | 0 | 2405 | 2428 | 4081 | 43 | 3 | 20 B |
| 24x4 | confidential signer | message bytes | none | - | - | - | - | - | 20 B |
| 24x4 | confidential in-proof | messages | 0 | 2501 | 2525 | 4080 | 42 | 2 | 19 B |
| 24x4 | confidential in-proof | message bytes | none | - | - | - | - | - | 19 B |
| 24x4 | anonymous hidden | reply | none | - | - | - | - | - | 213 B |
| 24x4 | anonymous hidden | messages | none | - | - | - | - | - | 213 B |
| 24x4 | anonymous hidden | message bytes | none | - | - | - | - | - | 248 B |
| 24x4 | anonymous public owners | reply | none | - | - | - | - | - | 342 B |
| 24x4 | anonymous public owners | messages | none | - | - | - | - | - | 342 B |
| 24x4 | anonymous public owners | message bytes | none | - | - | - | - | - | 377 B |
| 32x2 | confidential signer | messages | none | - | - | - | - | - | 80 B |
| 32x2 | confidential signer | message bytes | none | - | - | - | - | - | 115 B |
| 32x2 | confidential in-proof | messages | none | - | - | - | - | - | 79 B |
| 32x2 | confidential in-proof | message bytes | none | - | - | - | - | - | 114 B |
| 32x2 | anonymous hidden | reply | none | - | - | - | - | - | 114 B |
| 32x2 | anonymous hidden | messages | none | - | - | - | - | - | 114 B |
| 32x2 | anonymous hidden | message bytes | none | - | - | - | - | - | 149 B |
| 32x2 | anonymous public owners | reply | none | - | - | - | - | - | 243 B |
| 32x2 | anonymous public owners | messages | none | - | - | - | - | - | 243 B |
| 32x2 | anonymous public owners | message bytes | none | - | - | - | - | - | 278 B |
| 40x2 | confidential signer | messages | none | - | - | - | - | - | 601 B |
| 40x2 | confidential signer | message bytes | none | - | - | - | - | - | 636 B |
| 40x2 | confidential in-proof | messages | none | - | - | - | - | - | 600 B |
| 40x2 | confidential in-proof | message bytes | none | - | - | - | - | - | 635 B |
| 40x2 | anonymous hidden | reply | none | - | - | - | - | - | 635 B |
| 40x2 | anonymous hidden | messages | none | - | - | - | - | - | 635 B |
| 40x2 | anonymous hidden | message bytes | none | - | - | - | - | - | 670 B |
| 40x2 | anonymous public owners | reply | none | - | - | - | - | - | 764 B |
| 40x2 | anonymous public owners | messages | none | - | - | - | - | - | 764 B |
| 40x2 | anonymous public owners | message bytes | none | - | - | - | - | - | 799 B |
