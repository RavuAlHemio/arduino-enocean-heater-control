# Load specified address space JSON, define the peripheral memory spaces and assign the peripheral struct types to the peripheral addresses.
#@encoding: utf-8
#@author Ondřej Hošek <ondra.hosek@gmail.com>
#@category 
#@keybinding 
#@menupath 
#@toolbar


import json
import os.path
from ghidra.app.services import DataTypeManagerService
from ghidra.program.model.data import PointerDataType
from ghidra.program.model.mem import MemoryConflictException
from ghidra.program.model.symbol import SourceType


class FixedData:
    def __init__(self, address, name, type_name, is_pointer=False):
        self.address = address
        self.name = name
        self.type_name = type_name
        self.is_pointer = is_pointer

    def __repr__(self):
        return "FixedData(address={0}, type_name={1})".format(
            repr(self.address),
            repr(self.type_name),
        )


class FixedIoMappedMemoryRange:
    def __init__(self, name, base_address, length):
        self.name = name
        self.base_address = base_address
        self.length = length

    def __repr__(self):
        return "FixedIoMappedMemoryRange(name={0}, base_address={1}, length={2})".format(
            repr(self.name),
            repr(self.base_address),
            repr(self.length),
        )


class CpuInfo:
    def __init__(self, name, fixed_data, fixed_io_mapped_memory_ranges, interrupt_offset):
        self.name = name
        self.fixed_data = fixed_data
        self.fixed_io_mapped_memory_ranges = fixed_io_mapped_memory_ranges
        self.interrupt_offset = interrupt_offset

    def __repr__(self):
        return "CpuInfo(name={0}, fixed_data={1}, fixed_io_mapped_memory_ranges={2})".format(
            repr(self.name),
            repr(self.fixed_data),
            repr(self.fixed_io_mapped_memory_ranges),
        )


# unfortunately, this information doesn't seem to exist in any SVD file
CPU_TO_KNOWN_FIXED_DATA = {
    "CM0plus": CpuInfo(
        name="CM0plus",
        fixed_data=[
            FixedData(
                address=0xE000E008,
                name="SCBpart0",
                type_name="SCBpart0",
            ),
            FixedData(
                address=0xE000E010,
                name="SysTick",
                type_name="SysTick",
            ),
            FixedData(
                address=0xE000E100,
                name="NVICpart0",
                type_name="NVICpart0",
            ),
            FixedData(
                address=0xE000ED00,
                name="SCBpart1",
                type_name="SCBpart1",
            ),
            FixedData(
                address=0xE000ED90,
                name="MPU",
                type_name="MPU",
            ),
            FixedData(
                address=0xE000EF00,
                name="NVICpart1",
                type_name="NVICpart1",
            ),
            FixedData(
                address=0x00080000,
                name="StackBase",
                type_name="void *",
            ),
            FixedData(
                address=0x00080004,
                name="EXC_Reset",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080008,
                name="EXC_NMI",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x0008000C,
                name="EXC_HardFault",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080010,
                name="EXC_MemMgmtFault",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080014,
                name="EXC_BusFault",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080018,
                name="EXC_UsageFault",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x0008002C,
                name="EXC_SVCall",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080030,
                name="EXC_DebugMonitor",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080038,
                name="EXC_PendSV",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x0008003C,
                name="EXC_SysTick",
                type_name="ExceptionHandler *",
            ),
        ],
        fixed_io_mapped_memory_ranges=[
            FixedIoMappedMemoryRange(
                name="CorePeripherals",
                base_address=0xE0000000,
                length=0x00100000,
            ),
        ],
        interrupt_offset=0x00080040,
    ),
    "CM3": CpuInfo(
        name="CM3",
        fixed_data=[
            FixedData(
                address=0xE000E008,
                name="SCBpart0",
                type_name="SCBpart0",
            ),
            FixedData(
                address=0xE000E010,
                name="SysTick",
                type_name="SysTick",
            ),
            FixedData(
                address=0xE000E100,
                name="NVICpart0",
                type_name="NVICpart0",
            ),
            FixedData(
                address=0xE000ED00,
                name="SCBpart1",
                type_name="SCBpart1",
            ),
            FixedData(
                address=0xE000ED90,
                name="MPU",
                type_name="MPU",
            ),
            FixedData(
                address=0xE000EF00,
                name="NVICpart1",
                type_name="NVICpart1",
            ),
            FixedData(
                address=0x00080000,
                name="StackBase",
                type_name="void *",
            ),
            FixedData(
                address=0x00080004,
                name="EXC_Reset",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080008,
                name="EXC_NMI",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x0008000C,
                name="EXC_HardFault",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080010,
                name="EXC_MemMgmtFault",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080014,
                name="EXC_BusFault",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080018,
                name="EXC_UsageFault",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x0008002C,
                name="EXC_SVCall",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080030,
                name="EXC_DebugMonitor",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x00080038,
                name="EXC_PendSV",
                type_name="ExceptionHandler *",
            ),
            FixedData(
                address=0x0008003C,
                name="EXC_SysTick",
                type_name="ExceptionHandler *",
            ),
        ],
        fixed_io_mapped_memory_ranges=[
            FixedIoMappedMemoryRange(
                name="CorePeripherals",
                base_address=0xE0000000,
                length=0x00100000,
            ),
        ],
        interrupt_offset=0x00080040,
    ),
}

def get_data_type_manager_by_name(state, name):
    tool = state.getTool()
    service = tool.getService(DataTypeManagerService)
    managers = service.getDataTypeManagers()
    for manager in managers:
        if manager.getName() == name:
            return manager
    return None

addr_space_java_file = askFile("Choose address space JSON file", "Open")
with open(addr_space_java_file.getAbsolutePath(), "rb") as f:
    addr_space = json.load(f)
device_name = os.path.splitext(addr_space_java_file.getName())[0]

program_listing = currentProgram.getListing()
symbol_table = currentProgram.getSymbolTable()
default_address_space = currentProgram.getAddressFactory().getDefaultAddressSpace()

# do we know the CPU?
cpu_info = CPU_TO_KNOWN_FIXED_DATA.get(addr_space["cpu_core"])
if cpu_info is not None:
    # write those definitions first
    cpu_data_type_manager = get_data_type_manager_by_name(state, cpu_info.name)
    if cpu_data_type_manager is None:
        raise ValueError("CPU data type manager {0} not found!".format(repr(cpu_info.name)))

    for memory_range in cpu_info.fixed_io_mapped_memory_ranges:
        try:
            overlay = False
            ghidra_range = currentProgram.memory.createUninitializedBlock(
                memory_range.name,
                default_address_space.getAddress(memory_range.base_address),
                memory_range.length,
                overlay,
            )
            ghidra_range.setRead(True)
            ghidra_range.setWrite(True)
            ghidra_range.setExecute(False)
            ghidra_range.setVolatile(True)
        except MemoryConflictException:
            print("Warning: \"{0}\" memory block already exists; hoping for the best...".format(memory_range.name))

    for datum in cpu_info.fixed_data:
        is_pointer = False
        if datum.type_name.endswith(" *"):
            datum.type_name = datum.type_name[:len(datum.type_name)-2]
            is_pointer = True
        datum_type = cpu_data_type_manager.getDataType("/{0}.h/{1}".format(
            cpu_info.name,
            datum.type_name,
        ))
        if is_pointer:
            datum_type = PointerDataType.getPointer(
                datum_type,
                cpu_data_type_manager,
            )
        if datum_type is None:
            print("failed to find type {0}".format(datum.type_name))
        clear_context = True
        program_listing.clearCodeUnits(
            default_address_space.getAddress(datum.address),
            default_address_space.getAddress(datum.address + datum_type.getLength()),
            clear_context,
        )
        program_listing.createData(
            default_address_space.getAddress(datum.address),
            datum_type,
        )
        symbol_table.createLabel(
            default_address_space.getAddress(datum.address),
            datum.name,
            SourceType.USER_DEFINED,
        )

# now, run through the device-specific stuff
try:
    overlay = False
    ghidra_range = currentProgram.memory.createUninitializedBlock(
        "Peripherals",
        default_address_space.getAddress(addr_space["peripheral_address_space"]["base_address"]),
        addr_space["peripheral_address_space"]["length"] + 1,
        overlay,
    )
    ghidra_range.setRead(True)
    ghidra_range.setWrite(True)
    ghidra_range.setExecute(False)
    ghidra_range.setVolatile(True)
except MemoryConflictException:
    print("Warning: \"Peripherals\" memory block already exists; hoping for the best...")

device_data_type_manager = get_data_type_manager_by_name(state, device_name)
if device_data_type_manager is None:
    raise ValueError("device data type manager {0} not found!".format(repr(device_name)))
for register in addr_space["registers"]:
    register_type = device_data_type_manager.getDataType("/{0}.h/{1}".format(
        device_name,
        register["type_name"],
    ))
    clear_context = True
    program_listing.clearCodeUnits(
        default_address_space.getAddress(register["address"]),
        default_address_space.getAddress(register["address"] + register_type.getLength()),
        clear_context,
    )
    program_listing.createData(
        default_address_space.getAddress(register["address"]),
        register_type,
    )
    symbol_table.createLabel(
        default_address_space.getAddress(register["address"]),
        register["register"],
        SourceType.USER_DEFINED,
    )

if cpu_info is not None:
    cpu_data_type_manager = get_data_type_manager_by_name(state, cpu_info.name)
    handler_type = cpu_data_type_manager.getDataType("/{0}.h/{1}".format(
        cpu_info.name,
        "ExceptionHandler",
    ))
    handler_pointer_type = PointerDataType.getPointer(
        handler_type,
        cpu_data_type_manager,
    )
    for interrupt in addr_space["interrupts"]:
        interrupt_address = cpu_info.interrupt_offset + 4 * interrupt["index"]
        clear_context = True
        program_listing.clearCodeUnits(
            default_address_space.getAddress(interrupt_address),
            default_address_space.getAddress(interrupt_address + 4),
            clear_context,
        )
        program_listing.createData(
            default_address_space.getAddress(interrupt_address),
            handler_pointer_type,
        )
        symbol_table.createLabel(
            default_address_space.getAddress(interrupt_address),
            "INT_" + interrupt["name"],
            SourceType.USER_DEFINED,
        )
