# Hardware compatibility (research notes)

Tags: **[V]** from a primary source in this session; **[U]** unverified memory.

## PS4 hardware families
| Family | Model codes | Southbridge | Notes |
|---|---|---|---|
| Original ("Phat") | CUH-10xx, 11xx, 12xx | Aeolia (early), Belize (later) [V] | Most Linux work started here. CUH-1215/1216 use Marvell 88w8897 Wi-Fi/BT (known to need patches) [V] |
| Slim | CUH-2xxx | Baikal / Belize [V] | The primary test console for this project (user-selected model; never auto-detected) |
| Pro | CUH-7xxx | Baikal [V] | |

SoC (all): AMD x86-64 "Jaguar" CPU cores with a GCN-generation GPU, GDDR5 shared memory **[U]**. Exact core/clock/revision differences between families were not verified here.

## Linux subsystem status per the community kernel README [V unless marked]
| Subsystem | Status in evidence |
|---|---|
| Kernel boots on Phat/Slim/Pro | Yes, specific kernels per model (Slim: 5.4.247 or 5.15.15; see `alternative-os-research.md`) |
| Wi-Fi / Bluetooth | Needs per-chip patches; some chips need firmware extracted from the console's own OS [V: README anchor "builtin-fw"] — a copyright/ToS consideration |
| GPU | amdgpu needs PS4-specific firmware; postmarketOS note "initramfs needs ps4 amdgpu firmware" [V]; acceleration quality [U] |
| Black screen at GUI login | A fixed issue on newer kernels [V] |
| Audio (HDMI), controller, power/sleep, fan, internal storage on every revision | Not verified here [U] |

## For PS4 Studio users
Nothing in the desktop app depends on any of this. The model dropdown is a user choice used for guidance text only.
