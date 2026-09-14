import {invoke, convertFileSrc} from "@tauri-apps/api/core";
import * as path from "@tauri-apps/api/path";

window.addEventListener("DOMContentLoaded", async () => {
	const input_file: HTMLElement = document.querySelector("#input_file")!;
	const input_image: HTMLImageElement = document.querySelector("#input_image")!;
	const output_image: HTMLImageElement = document.querySelector("#output_image")!;
	const output_button: HTMLElement = document.querySelector("#output_button")!;
	const scale_input: HTMLInputElement = document.querySelector("#scale")!;
	let input_src: string | null = null;
	let scale = parseInt(scale_input.value);
	const output_src = await path.join(await path.homeDir(), 'Desktop', `minecraft_pixelart_${new Date().toJSON()}.jpg`);

	scale_input.addEventListener("input", e => scale = parseInt((e.target as HTMLInputElement).value));

	input_file.addEventListener("click", async e => {
		e.preventDefault();
		input_src = await invoke('choose_file')
		if(input_src !== null)
			input_image.src = convertFileSrc(input_src);
	});

	output_button.addEventListener("click", async () => {
		if(input_src === null) {
			alert("No input file given");
		} else {
			await invoke('get_image', {input_name: input_src, output_name: output_src, scale: scale});
			output_image.removeAttribute('src');
			output_image.src = convertFileSrc(output_src);
		}
	})
});