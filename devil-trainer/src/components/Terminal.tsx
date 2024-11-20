import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { Button } from "./Button";
import { useConsoleStore } from "../stores/console";
import { BaseDirectory, create } from "@tauri-apps/plugin-fs";
import { SerialPort } from "tauri-plugin-serialplugin";
import * as os from "@tauri-apps/plugin-os";

export const Terminal = ({ className, ...props }: { className: string }) => {
	const consoleState = useConsoleStore((state) => state);

	const [serialPort, setSerialPort] = useState<SerialPort | undefined>();

	// Handle Start and Restart buttons
	// Handle Start button
	const connect = async () => {
		await SerialPort.closeAll();
		const ports = await SerialPort.available_ports();

		for (const [name, info] of Object.entries(ports)) {
			if (info.manufacturer != 'Devils Prosthetics') continue;

			if (os.platform() == "macos" && name.includes('tty')) continue;

			console.log(`connecting to ${name}`)

			const serialPort = new SerialPort({
				path: name,
				baudRate: 115200
			});

			await serialPort.open();

			setSerialPort(serialPort);

			break;
		}
	}
	// Handle Restart button (no backend call, just resets the console)
	const handleRestart = () => consoleState.set(['Restart clicked']);

	useEffect(() => {
		if (!serialPort) {
			connect();
		}
	}, [serialPort]);

	return (
		<div className={`relative h-full ${className}`} {...props}>
			<div className="absolute right-3">
				<div className='flex space-x-2'>
					<Button className='w-fit' onClick={handleRestart}>
						Restart
					</Button>
					<Button className='w-fit' onClick={handleStart}>Start</Button>
				</div>
			</div>
			<div className="overflow-y-scroll h-full">
				{consoleState.output.map((line, index) => {
					// Output each line as a div.
					return (<div key={index}>{line}</div>)
				})}
			</div>
		</div>
	);
}
