.yolo:
    ldi     $61
.start:
    st      [CHAR]
    print   [STR]
    subi    $7a
    jz      .end
    ld      [CHAR]
    addi    $01
    jmp     .start
.end:
    stop

.STR:
    .byte   $1
.CHAR:
    .byte   $0   ; will be overwritten
