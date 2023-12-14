.assembly asm{}
.assembly extern System.Runtime.InteropServices{.ver 6:12:0:0 }
.assembly extern System.Runtime{.ver 6:12:0:0 }


.class public Unresolved extends [System.Runtime]System.ValueType{
}

.class private _ZN4core3ffi6c_void17hdc059d755059b48fE extends [System.Runtime]System.ValueType{
}

.class public RustStr extends [System.Runtime]System.ValueType{
}

.class public Foreign extends [System.Runtime]System.ValueType{
}

.class public RustVoid extends [System.Runtime]System.ValueType{
}
.method private hidebysig static valuetype _ZN4core3ffi6c_void17hdc059d755059b48fE* malloc(native uint){
	.locals (

	)
	ldarg.0
	call  native int  [System.Runtime.InteropServices]System.Runtime.InteropServices.Marshal:: AllocHGlobal(native int)
	ret
}
.method private hidebysig static void puts(uint8*){
	.locals (
		[0] uint8
	)
	bb_0:
	ldarg.0
	ldind.i1
	stloc.0
	ldloc.0
	ldc.i4.0
	beq bb_1
	ldloc.0
	conv.i
	call  void  [System.Console]System.Console:: Write(char)
	ldarg.0
	ldc.i4.1
	add
	starg.s 0
	br bb_0
	bb_1:
	ret
}
.method private hidebysig static uint8* __rust_alloc(native uint,native uint){
	.locals (

	)
	ldarg.0
	ldarg.1
	call  void*  [System.Runtime.InteropServices]System.Runtime.InteropServices.NativeMemory:: AlignedAlloc(native uint,native uint)
	ret
}
.method private hidebysig static valuetype _ZN4core3ffi6c_void17hdc059d755059b48fE* realloc(valuetype _ZN4core3ffi6c_void17hdc059d755059b48fE*,native uint){
	.locals (

	)
	ldarg.0
	ldarg.1
	call  native int  [System.Runtime.InteropServices]System.Runtime.InteropServices.Marshal:: ReAllocHGlobal(native int,native int)
	ret
}
.method public hidebysig static int32 andi32(int32,int32){
	.locals (
		[0] int32
	)
	bb_0:
	ldarg.0
	ldarg.1
	and
	stloc.0
	ldloc.0
	ret
}
.method private hidebysig static float32 sqrtf32(float32){
	.locals (

	)
	ldarg.0
	ret
}
.method public hidebysig static int32 remi32(int32,int32){
	.locals (
		[0] int32,
		[1] bool,
		[2] bool,
		[3] bool,
		[4] bool
	)
	bb_0:
	ldarg.1
	ldc.i4.0
	ceq
	stloc.1
	br bb_1
	bb_1:
	ldarg.1
	ldc.i4.m1
	ceq
	stloc.2
	ldarg.0
	ldc.i4 -2147483648
	ceq
	stloc.3
	ldloc.2
	ldloc.3
	and
	stloc.s 4
	br bb_2
	bb_2:
	ldarg.0
	ldarg.1
	rem
	stloc.0
	ldloc.0
	ret
}
.method public hidebysig static bool eq(int32,int32){
	.locals (
		[0] bool
	)
	bb_0:
	ldarg.0
	ldarg.1
	ceq
	stloc.0
	ldloc.0
	ret
}
.method private hidebysig static void free(valuetype _ZN4core3ffi6c_void17hdc059d755059b48fE*){
	.locals (

	)
	ldarg.0
	call  void  [System.Runtime.InteropServices]System.Runtime.InteropServices.Marshal:: FreeHGlobal(native int)
	ret
}
.method public hidebysig static int32 muli32(int32,int32){
	.locals (
		[0] int32
	)
	bb_0:
	ldarg.0
	ldarg.1
	mul
	stloc.0
	ldloc.0
	ret
}
.method public hidebysig static int32 shli32(int32,int32){
	.locals (
		[0] int32
	)
	bb_0:
	ldarg.0
	ldarg.1
	shl
	stloc.0
	ldloc.0
	ret
}
.method public hidebysig static bool neq(int32,int32){
	.locals (
		[0] bool
	)
	bb_0:
	ldarg.0
	ldarg.1
	ceq
	ldc.i4.0
	ceq
	stloc.0
	ldloc.0
	ret
}
.method public hidebysig static float32 negate_f32(float32){
	.locals (
		[0] float32
	)
	bb_0:
	ldarg.0
	neg
	stloc.0
	ldloc.0
	ret
}
.method public hidebysig static int32 divi32(int32,int32){
	.locals (
		[0] int32,
		[1] bool,
		[2] bool,
		[3] bool,
		[4] bool
	)
	bb_0:
	ldarg.1
	ldc.i4.0
	ceq
	stloc.1
	br bb_1
	bb_1:
	ldarg.1
	ldc.i4.m1
	ceq
	stloc.2
	ldarg.0
	ldc.i4 -2147483648
	ceq
	stloc.3
	ldloc.2
	ldloc.3
	and
	stloc.s 4
	br bb_2
	bb_2:
	ldarg.0
	ldarg.1
	div
	stloc.0
	ldloc.0
	ret
}
.method private hidebysig static uint8 volatile_load(uint8*){
	.locals (

	)
	ldarg.0
	ldind.i1
	ret
}
.method public hidebysig static int32 negate_i32(int32){
	.locals (
		[0] int32
	)
	bb_0:
	ldarg.0
	neg
	stloc.0
	ldloc.0
	ret
}
.method public hidebysig static int32 shri32(int32,int32){
	.locals (
		[0] int32
	)
	bb_0:
	ldarg.0
	ldarg.1
	shr
	stloc.0
	ldloc.0
	ret
}
.method private hidebysig static bool unlikely(bool){
	.locals (

	)
	ldarg.0
	ret
}
.method public hidebysig static int32 ori32(int32,int32){
	.locals (
		[0] int32
	)
	bb_0:
	ldarg.0
	ldarg.1
	or
	stloc.0
	ldloc.0
	ret
}
.method private hidebysig static void abort(){
	.locals (

	)
	ldstr "Called abort!"
	newobj instance void [System.Runtime]System.Exception::.ctor(string)
	throw
	ret
}
.method public hidebysig static int32 pow2(int32){
	.locals (
		[0] int32
	)
	bb_0:
	ldc.i4.1
	ldarg.0
	shl
	stloc.0
	ldloc.0
	ret
}
.method private hidebysig static void __rust_dealloc(uint8*,native uint,native uint){
	.locals (

	)
	ldarg.0
	call  void  [System.Runtime.InteropServices]System.Runtime.InteropServices.NativeMemory:: AlignedFree(valuetype RustVoid*)
	ret
}
.method public hidebysig static int32 sqr_mag(int32,int32){
	.locals (
		[0] int32,
		[1] int32,
		[2] int32
	)
	bb_0:
	ldarg.0
	ldarg.0
	mul
	stloc.1
	ldarg.1
	ldarg.1
	mul
	stloc.2
	ldloc.1
	ldloc.2
	add
	stloc.0
	ldloc.0
	ret
}
.method public hidebysig static int32 xori32(int32,int32){
	.locals (
		[0] int32
	)
	bb_0:
	ldarg.0
	ldarg.1
	xor
	stloc.0
	ldloc.0
	ret
}
.method public hidebysig static int32 subi32(int32,int32){
	.locals (
		[0] int32
	)
	bb_0:
	ldarg.0
	ldarg.1
	sub
	stloc.0
	ldloc.0
	ret
}
.field static uint8** environ
.field static uint8 __rust_alloc_error_handler_should_panic
.field static uint8 __rust_no_alloc_shim_is_unstable
