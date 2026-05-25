; ModuleID = 'test_project'
source_filename = "test_project"

@0 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@1 = private unnamed_addr constant [3 x i8] c"%d\00", align 1

declare i32 @printf(i32, ...)

declare i32 @scanf(i32, ...)

define void @main() {
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.body, %0
  %1 = load i32, ptr %i, align 4
  %2 = icmp slt i32 %1, 10
  br i1 %2, label %while.body, label %while.end

while.body:                                       ; preds = %while.cond
  %3 = load i32, ptr %i, align 4
  %4 = call i32 (i32, ...) @printf(ptr @0, i32 %3)
  %5 = call i32 (i32, ...) @scanf(ptr @1, ptr %i)
  br label %while.cond

while.end:                                        ; preds = %while.cond
  ret i32 0
  ret void
}
