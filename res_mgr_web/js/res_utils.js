// 直接调用，res上可能没有destroy方法，调用会出错，所以封装出来
function destroy_res(res) {
	res.destroy && res.destroy();
}