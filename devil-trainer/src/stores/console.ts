import { create } from 'zustand'

// Creates a store to hold the Console's output
interface ConsoleState {
	output: string[],
	clear: () => void,
	add: (item: string) => void,
	set: (output: string[]) => void,
}

// Defines all of the functions for the store
export const useConsoleStore = create<ConsoleState>((set) => ({
	output: ['Console output will show up here'],
	set: (output: string[]) => set({ output }),
	clear: () => set({ output: [] }),
	add: (item: string) => set((state) => {
		state.output.push(item);
		return {
			output: state.output
		}
	})
}))
