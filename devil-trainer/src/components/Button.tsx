interface ButtonProps extends React.ButtonHTMLAttributes<HTMLElement> {
	makeDiv?: boolean
}

// A button component, just gives consistent styling
export const Button = ({className, children, makeDiv, ...props}: ButtonProps) => {
	if (makeDiv == false) {
		return (
			<button className={`select-none cursor-pointer bg-rose-500 hover:bg-rose-700 text-white font-bold py-2 px-4 rounded ${className}`} {...props}>
				{ children }
			</button>
		);
	}

	return (
		<div className={`select-none cursor-pointer bg-rose-500 hover:bg-rose-700 text-white font-bold py-2 px-4 rounded ${className}`} {...props}>
			{ children }
		</div>
	);
}
