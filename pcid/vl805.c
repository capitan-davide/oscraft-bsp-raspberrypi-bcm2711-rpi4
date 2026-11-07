/*
 * The Raspberry Pi 4's USB 3 controller, a VIA VL805 on the PCIe bus,
 * keeps its firmware in RAM: once the host bridge is reset (pcid's
 * bcm2838_pci.c does), the VideoCore firmware must load it again, which it
 * does when told through the mailbox's NOTIFY_XHCI_RESET property, with the
 * device's PCI address (bus << 20 | slot << 15 | function << 12: 0x100000
 * for 1:0:0). FreeBSD does it in its xhci attachment (bcm2838_xhci.c);
 * QNX's BSP before io-usb-otg starts (mbox-bcm2711 notifyxhcireset=0x100000).
 *
 * Here it is pcid's: a driver for the VL805 on pcid's PCI bus whose attach
 * makes FreeBSD's mailbox call (bcm2835_mbox.c, in the same module) unless
 * the firmware is already there, and then leaves the device to the server
 * that drives it (usbd, through its PCI client bus).
 */
#include <sys/param.h>
#include <sys/systm.h>
#include <sys/bus.h>
#include <sys/kernel.h>
#include <sys/module.h>

#include <dev/pci/pcireg.h>
#include <dev/pci/pcivar.h>

#include <arm/broadcom/bcm2835/bcm2835_mbox_prop.h>

#define	VL805_VENDOR		0x1106
#define	VL805_DEVICE		0x3483
/* The firmware's revision, 0 (or all ones) when there is none. */
#define	VL805_FIRMWARE_REG	0x50

static uint32_t
vl805_firmware(device_t dev)
{
	uint32_t rev = pci_read_config(dev, VL805_FIRMWARE_REG, 4);

	return (rev == 0xffffffff ? 0 : rev);
}

static int
vl805_probe(device_t dev)
{
	if (pci_get_vendor(dev) != VL805_VENDOR ||
	    pci_get_device(dev) != VL805_DEVICE)
		return (ENXIO);
	device_set_desc(dev, "VL805 USB 3.0 controller (firmware loader)");
	return (BUS_PROBE_DEFAULT);
}

static int
vl805_attach(device_t dev)
{
	uint32_t addr, rev;
	int error;

	rev = vl805_firmware(dev);
	if (rev == 0) {
		addr = pci_get_bus(dev) << 20 | pci_get_slot(dev) << 15 |
		    pci_get_function(dev) << 12;
		error = bcm2835_mbox_notify_xhci_reset(addr);
		if (error != 0)
			device_printf(dev, "firmware load failed: %d\n", error);
		DELAY(1000);
		rev = vl805_firmware(dev);
	}
	if (rev == 0)
		device_printf(dev, "no firmware\n");
	else
		device_printf(dev, "firmware revision %x\n", rev);
	return (0);
}

static device_method_t vl805_methods[] = {
	DEVMETHOD(device_probe,		vl805_probe),
	DEVMETHOD(device_attach,	vl805_attach),
	DEVMETHOD_END
};

static driver_t vl805_driver = { "vl805", vl805_methods, 0 };
DRIVER_MODULE(vl805, pci, vl805_driver, 0, 0);
