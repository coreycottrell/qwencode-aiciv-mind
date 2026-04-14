#!/bin/bash

fibonacci() {
    local n=$1
    local a=0
    local b=1
    local c=0
    
    for (( i=0; i<n; i++ )); do
        c=$((a + b))
        a=$b
        b=$c
    done
    
    echo $a
}

time fibonacci 30 > fibonacci_result.txt
