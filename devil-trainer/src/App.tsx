import 'react-toastify/dist/ReactToastify.css';
import { ToastContainer } from 'react-toastify';

import { Terminal } from './components/Terminal';
import { Logo } from './components/Logo';
import { FileSelector } from './components/FileSelector';
import { Checkpoint } from './components/Checkpoint';

// Displays the app
const App = () => {
	return (
		<>
			<div className='flex flex-col justify-center items-center p-10 h-screen'>
				<Logo size={'100'} />
				<div className='flex py-3 space-x-3'>
					<Checkpoint className='bg-pink-900 rounded-xl p-3 w-full' />
					<FileSelector className='bg-pink-900 rounded-xl p-3 flex flex-col justify-center' />
				</div>
				<div className='grow overflow-hidden w-full'>
					<Terminal className='bg-pink-900 rounded-xl p-3 w-full' />
				</div>
			</div>
			<ToastContainer />
		</>
	);
};

export default App;
