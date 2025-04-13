# GPU Info Doc
> Linux,Windows

## Пример использования
```rust
let gpu = gpu_info::get();

"Vendor: {:?}", gpu.vendor()
"Name: {:?}", gpu.name_gpu()
"Utilization: {:?}", gpu.utilization()
"Temperature: {:?}", gpu.temperature()
"Clock Speed: {:?}", gpu.core_clock()
"Power Usage: {:?}", gpu.power_usage()
"Memory Usage: {:?}", gpu.memory_util()
"Memory Total: {:?}", gpu.memory_total()
"Is active: {:?}", gpu.active()
```

##
```
cargo run --example cli
```
