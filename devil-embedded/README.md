# Devil Embedded

The Devil Embedded program is a Rust application specifically designed for the Raspberry Pi Pico. The main goal of this program is to gather sensor data, process it through a machine learning model, make a prediction on which gesture was made, and use that gesture to control servos. The program helps set up interrupt handlers for peripherals like USB, Analog-to-Digital Converter (ADC), and Programmable I/O (PIO). The program defines servo motors and sets up their configurations to be able to perform movements like thumb rotations, finger gestures, and arm positioning.

## Over-Arching View

The data is collected through ADC channels connected to Myoelectric (EMG) sensors, and each input is normalized between 0 and 1 for consistency. After normalization, the inputs are transformed into tensors, which the machine learning model processes using the NdArray backend. The model outputs probabilities for various gestures using the infer function from the devil-ml crate, which are further normalized using a softmax function. The gesture with the highest probability is then displayed and can be used to control servo motors to mimic the predicted gesture.

The program uses the embassy framework, which is optimized for low-power embedded devices. Future changes could include adding more gestures or improving the servo’s responsiveness to model predictions.
