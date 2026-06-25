# Mastermind

Command line implementation of the code-breaking game [Mastermind(https://en.wikipedia.org/wiki/Mastermind_(board_game))] with an implementation to brute-force an optimal solution strategy.

Main assumptions:
* Permutation: If two codes generate the same result just one needs to be considered (i.e. for a given solution the same number of codes related with the same grade to that solution).
* Pruning: The theoretically fastest solution for $n$ codes would be, if all codes can be identified with $2 \cdot n - 1$ tries (if one input creates one unique grade for each code left). This allows to estimate certain solution branches and cut them off in case the number exceeds an already given solution path.