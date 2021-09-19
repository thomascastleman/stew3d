label1:
    mvi 1, c
    jmp label3
    call label1
label2:
label3:
    mov a, b
    hlt
    jae label2
label4: