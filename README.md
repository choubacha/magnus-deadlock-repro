# Magnus Deadlock

This small application can be used to demonstrate a deadlock within the
initialization of the ruby extention objects. It happens when a rust-extension
object is initialized for the first time on a thread at the same time as on
other threads. When this happens the entire ruby run-time is blocked.

The reason this happens is due to how rb_protect works when calling into a
closure. When it calls out it may choose to yield control back to ruby which may
then activate a different thread. rb_protect is called when initialization the
lazy type in magnus is called. This can create a competition for two locks. One
is the Once lock that the code block is running and the other lock is the GVL.

You can see the callstack using LLDB when running the specs. There is only a
single spec and it simulates the conditions of the deadlock. It does not always
hit so you may need to run it multiple times.

Here are the traces of the locked threads:

```
  thread #102, name = 'deadlock_spec.rb:8'
    frame #0: 0x00000001865f19ec libsystem_kernel.dylib`__psynch_cvwait + 8
    frame #1: 0x000000018662f55c libsystem_pthread.dylib`_pthread_cond_wait + 1228
    frame #2: 0x0000000104fec838 libruby.3.1.dylib`gvl_acquire_common + 300
    frame #3: 0x0000000104fe36b4 libruby.3.1.dylib`rb_thread_schedule_limits + 496
    frame #4: 0x0000000104fe4448 libruby.3.1.dylib`rb_threadptr_execute_interrupts + 180
    frame #5: 0x0000000105028d8c libruby.3.1.dylib`vm_call0_body + 2460
    frame #6: 0x0000000105040ba8 libruby.3.1.dylib`rb_call0 + 812
    frame #7: 0x0000000104e85f2c libruby.3.1.dylib`rb_protect + 356
    frame #8: 0x0000000104bdeec4 deadlock.bundle`magnus::value::ReprValue::funcall::h4d22baf132a5cbd7 + 128
    frame #9: 0x0000000104bdf130 deadlock.bundle`core::ops::function::FnOnce::call_once::ha5b33cc778bdfd99 + 60
    frame #10: 0x0000000104bde7e4 deadlock.bundle`std::sync::once::Once::call_once::_$u7b$$u7b$closure$u7d$$u7d$::hcac4b3e0db9005a4 + 36
    frame #11: 0x0000000104c16d38 deadlock.bundle`std::sys_common::once::queue::Once::call::h6ae0db728dbc12a8 + 736
    frame #12: 0x0000000104bde950 deadlock.bundle`std::panicking::try::hbc2fa018473f07a7 + 316
    frame #13: 0x0000000104bdf914 deadlock.bundle`deadlock::init::anon::h01225d0719319880 + 44
    frame #14: 0x000000010503b4c0 libruby.3.1.dylib`vm_call_cfunc_with_frame + 232
    frame #15: 0x000000010503d80c libruby.3.1.dylib`vm_sendish + 320
    frame #16: 0x000000010501df4c libruby.3.1.dylib`vm_exec_core + 10556
    frame #17: 0x0000000105032660 libruby.3.1.dylib`rb_vm_exec + 2688
    frame #18: 0x00000001050303a0 libruby.3.1.dylib`rb_vm_invoke_proc + 1208
    frame #19: 0x0000000104fec64c libruby.3.1.dylib`thread_do_start_proc + 688
    frame #20: 0x0000000104febe2c libruby.3.1.dylib`thread_start_func_2 + 1184
    frame #21: 0x0000000104feb818 libruby.3.1.dylib`thread_start_func_1 + 264
    frame #22: 0x000000018662ef94 libsystem_pthread.dylib`_pthread_start + 136
```

```
  thread #3, name = 'deadlock_spec.rb:8'
    frame #0: 0x00000001865ee170 libsystem_kernel.dylib`semaphore_wait_trap + 8
    frame #1: 0x000000018647e984 libdispatch.dylib`_dispatch_sema4_wait + 28
    frame #2: 0x000000018647f034 libdispatch.dylib`_dispatch_semaphore_wait_slow + 132
    frame #3: 0x0000000104c16cdc deadlock.bundle`std::sys_common::once::queue::Once::call::h6ae0db728dbc12a8 + 644
    frame #4: 0x0000000104bde950 deadlock.bundle`std::panicking::try::hbc2fa018473f07a7 + 316
    frame #5: 0x0000000104bdf914 deadlock.bundle`deadlock::init::anon::h01225d0719319880 + 44
    frame #6: 0x000000010503b4c0 libruby.3.1.dylib`vm_call_cfunc_with_frame + 232
    frame #7: 0x000000010503d80c libruby.3.1.dylib`vm_sendish + 320
    frame #8: 0x000000010501df4c libruby.3.1.dylib`vm_exec_core + 10556
    frame #9: 0x0000000105032660 libruby.3.1.dylib`rb_vm_exec + 2688
    frame #10: 0x00000001050303a0 libruby.3.1.dylib`rb_vm_invoke_proc + 1208
    frame #11: 0x0000000104fec64c libruby.3.1.dylib`thread_do_start_proc + 688
    frame #12: 0x0000000104febe2c libruby.3.1.dylib`thread_start_func_2 + 1184
    frame #13: 0x0000000104feb818 libruby.3.1.dylib`thread_start_func_1 + 264
    frame #14: 0x000000018662ef94 libsystem_pthread.dylib`_pthread_start + 136
```

The key part of these stacks is that one is waiting on the semaphore for the
Once::call while the other is waiting on the GVL and already but holds the
semaphore lock. The part of code the yields control back to ruby is the call to
`rb_thread_schedule_limits`.
