import { useState } from "react";
import { Button } from "./Button";
import { invoke } from "@tauri-apps/api/core";
import { InputFile } from "./InputFile";

// Define data structure
type Point = {
	name: string,
	value: number
}

export const Checkpoint = ({ className, ...props }: { className?: string }) => {
	const [data, setData] = useState<Point[]>([
		{ name: 'Point 1', value: 10 },
		{ name: 'Point 2', value: 30 },
		{ name: 'Point 3', value: 20 },
		{ name: 'Point 4', value: 80 },
		{ name: 'Point 5', value: 60 },
	]);
	const [selectedPoint, setSelectedPoint] = useState<Point | null>(null);

	// Handle dot click (select a data point)
	const handleDotClick = (point: Point) => {
		setSelectedPoint(point);
	};

	const handleFolderSelect = async (event: React.ChangeEvent<HTMLInputElement>) => {
		const files = event.target.files;
		if (!files || files.length === 0) return;
		const file = files[0];
		const reader = new FileReader();
	
		reader.onload = (e) => {
			const text = e.target?.result as string;
			const newData = text
				.split("\n")
				.filter(line => line.trim() !== "")
				.map((line, index) => {
					const [value] = line.split(","); // Get the first part of the line
					return {
						name: `Point ${index + 1}`,
						value: Number(value.trim()) // Convert the value to a number
					};
				});
			setData(newData);
		};
	
		reader.readAsText(file);
	};
	
	return (
		<div {...props} className={`flex flex-col justify-center items-center ${className}`} style={{ maxHeight: "350px", width: "800px" }}>
			{/* Table Container with Scroll */}
			<div className="overflow-auto" style={{ maxHeight: "150px" }}>
				<table className="bg-pink-950 p-4 rounded-xl border-separate border-spacing-x-5 whitespace-nowrap">
					<thead>
						<tr>
							<th>Name</th>
							<th>Value</th>
						</tr>
					</thead>
					<tbody>
						{data.map((data, index) => (
							<tr key={index}>
								<td className="text-center">{data.name}</td>
								<td className="text-center">{data.value}</td>
							</tr>
						))}
					</tbody>
				</table>
			</div>

			<div className='flex flex-col w-full justify-center items-center mx-2'>
				{/* Line Graph Container with Scroll */}
				<div style={{ maxHeight: "150px", width: "100%", overflow: 'hidden' }}>
					<svg className='w-full h-full' viewBox='-20 0 7500 100' style={{ height: "100px", marginLeft: "10px"}}>
						<polyline
							fill="none"
							stroke="white"
							strokeWidth="10" // Adjust strokeWidth for visibility
							points={data.map((data, index) => `${index * 100 + 5},${100 - data.value}`).join(' ')}
						/>
						{data.map((data, index) => (
							<circle
								key={index}
								cx={index * 100 + 5}
								cy={100 - data.value}
								r="25"
								fill={selectedPoint === data ? 'red' : 'white'}
								stroke="black"
								strokeWidth="5" // Adjust strokeWidth for visibility
								cursor="pointer"
								onClick={() => handleDotClick(data)}
							/>
						))}
					</svg>
				</div>

				<div>
					<span>
						{selectedPoint ? `Selected: ${selectedPoint.name} - Value: ${selectedPoint.value}` : 'No data point selected'}
					</span>
				</div>

				{/* Load Model Button */}
				<InputFile onChange={handleFolderSelect} className="mt-2" />
			</div>
		</div>
	);
};
