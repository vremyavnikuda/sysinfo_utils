# GPU Info Doc
> Linux, Windows, MacOS

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

## Пример использования форматированного результата
```rust
"Formated Name Gpu: {}",gpu.format_name_gpu()
"Formated Temperature GPU: {}",gpu.format_temperature()
"Formated Power Usage GPU: {}",gpu.format_power_usage()
"Formated Core Clock GPU: {}",gpu.format_core_clock()
"Formated Memory Util GPU: {}",gpu.format_memory_util()
"Formated Active GPU: {}",gpu.format_active()
"Formated Power Limit GPU: {}",gpu.format_power_limit()
"Formated Memory Total GPU: {}",gpu.format_memory_total()
"Formated Driver Version GPU: {}",gpu.format_driver_version()
"Formated Max Clock Speed GPU: {}",gpu.format_max_clock_speed()
```
##
```
cargo run --example cli
```
