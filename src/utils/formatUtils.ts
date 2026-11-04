export const DISPLAY_MAP: Record<string, string> = {
	CmdOrCtrl: 'Ctrl',
	Ctrl: 'Ctrl',
	Control: 'Ctrl',
	Shift: 'Shift',
	Alt: 'Alt',
	Meta: 'Cmd',
	Super: 'Win',
	Win: 'Win',
	Space: 'Space',
	Return: 'Enter',
};

export function formatShortcut(shortcut: string): string {
	return shortcut
		.split('+')
		.map((part) => DISPLAY_MAP[part] ?? part)
		.join('+');
}
