# svd2ghidrac

Converts SVD (System View Description) files, which describe the registers of ARM-based chips, to C headers parseable by Ghidra.

This simplifies the process of teaching Ghidra's decompiler about your chip's memory-mapped I/O registers, as it can generate data type libraries from the parsed C headers.
