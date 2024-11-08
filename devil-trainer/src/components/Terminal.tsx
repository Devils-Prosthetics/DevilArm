import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { Button } from "./Button";
import { useConsoleStore } from "../stores/console";
import { BaseDirectory, create } from "@tauri-apps/plugin-fs";


export const Terminal = ({ className, ...props }: { className: string }) => {
	const consoleState = useConsoleStore((state) => state);

	// Handle Start and Restart buttons
	// Handle Start button
	const handleStart = async () => {
		consoleState.set(['Starting serial port connection...']);
		try {
			const response = await invoke('serial_start', { port: '/dev/ttyACM0' }) as string;
			consoleState.set([response]);

		} catch (error) {
			consoleState.set([`Error: ${error}`]);
		}
	};

	// Handle Restart button (no backend call, just resets the console)
	const handleRestart = () => consoleState.set(['Restart clicked']);

	useEffect(() => {
		let logToCSV = false;
		let holdData = "";
		let fileNum = 0;
		// Add in any serial-data events that are called from rust to the console
		const unlisten = listen('serial-data', async (event) => {
			const payload = event.payload as string;
			if (payload.includes("NewData") === true) {
				logToCSV = true;
				consoleState.add(payload);
				return;
			}
			if (payload.includes("EndData") === true) {
				logToCSV = false;
				return;
			}
			if (logToCSV === true) {
				holdData = holdData + payload;
			} else {
				if (holdData != "") {
					// Export to CSV
					const file = await create('GitHub/DevilArm/devil-ml/training/data/temp' + fileNum + '.csv', { baseDir: BaseDirectory.Document });
					await file.write(new TextEncoder().encode(holdData));
					await file.close();
					fileNum++;
				}
				holdData = "";
				consoleState.add(payload);
			}
		});



		return () => {
			// remove the listener on  unmount
			unlisten.then((fn) => fn());
		};
	}, []);

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
