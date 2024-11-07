import { Button } from "./Button";

// Creates an input file button
export const InputFile = ({className, onChange}: {className?: string, onChange: React.ChangeEventHandler<HTMLInputElement>}) => {
	return (
		<div className={className}>
			<label htmlFor='file-upload'>
				<Button makeDiv={true}>
					Custom Upload
				</Button>
			</label>
			<input id='file-upload' type='file' onChange={onChange} />
		</div>
	);
}
